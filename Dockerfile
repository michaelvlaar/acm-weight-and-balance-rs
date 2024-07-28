FROM rust:latest as builder
WORKDIR /usr/src/acm_weight_and_balance
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim 
RUN apt-get update && apt-get install -y \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/acm_weight_and_balance /usr/local/bin/acm_weight_and_balance
CMD ["acm_weight_and_balance"]
