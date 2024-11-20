FROM rust:bookworm AS builder

ARG RUST_LOG=info
ENV RUST_LOG=${RUST_LOG}

ARG POLL_PERIOD=60
ENV POLL_PERIOD=${POLL_PERIOD}

WORKDIR /usr/src/bitcoin_ath
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt update && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/bitcoin_ath/target/release/bitcoin_ath /usr/local/bin/bitcoin_ath

ENTRYPOINT ["/usr/local/bin/bitcoin_ath"]
