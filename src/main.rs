use serde_json::{self};
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

const POLL_PERIOD: u64 = 30;
const URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=USD&include_last_updated_at=true&include_24hr_change=true";
const NOTIFICATION_URL: &str = "https://ntfy.sh/bitcoin_ath";

#[tokio::main]
async fn main() -> Result<(), JobSchedulerError> {
    env_logger::init();
    log::info!("Bitcoin ATH starting.");

    let poll_period_str = std::env::var("POLL_PERIOD").unwrap_or(POLL_PERIOD.to_string());
    let poll_period = poll_period_str.parse::<u64>().unwrap();
    log::info!("Poll period: {} seconds", poll_period);

    let mut sched = JobScheduler::new().await?;

    let job = Job::new_repeated_async(Duration::from_secs(poll_period), |_uuid, _l| {
        Box::pin(async {
            check_ath().await;
        })
    })?;

    sched.add(job).await?;
    sched.start().await?;

    loop {
        let next = sched.time_till_next_job().await?;
        let dur = next.unwrap_or_default();

        std::thread::sleep(dur);
    }
}

fn load_last_ath_from_file() -> u64 {
    let contents = std::fs::read_to_string("last_ath.txt");
    match contents {
        Ok(val) => val.parse::<u64>().unwrap(),
        Err(_) => 0,
    }
}

fn save_last_ath_to_file(ath: u64) {
    std::fs::write("last_ath.txt", ath.to_string()).unwrap();
}

async fn send_notification(ath: u64) {
    let message = format!("New bitcoin all time high: ${}", ath);

    let client = reqwest::Client::new();
    let res = client.post(NOTIFICATION_URL).body(message).send().await;

    match res {
        Ok(_) => log::info!("Notification sent for new ATH ({})", ath),
        Err(err) => log::error!("Error sending notification: {}", err),
    }
}

async fn check_ath() {
    match reqwest::get(URL).await {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.unwrap();

            let bitcoin_last_value = json["bitcoin"]["usd"].as_u64().unwrap_or(0);
            log::debug!("Bitcoin last values: {}", bitcoin_last_value);

            let last_ath = load_last_ath_from_file();

            if last_ath < bitcoin_last_value {
                log::info!("New ATH: {}", bitcoin_last_value);
                save_last_ath_to_file(bitcoin_last_value);
                send_notification(bitcoin_last_value).await;
            }

            bitcoin_last_value
        }
        Err(err) => {
            log::error!("Error requesting data: {}", err);
            0
        }
    };
}
