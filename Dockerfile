FROM clux/muslrust:1.76.0-stable AS chef
WORKDIR /carbonara-watchdog
RUN cargo install cargo-chef

FROM chef AS planner
COPY Cargo.toml Cargo.lock src ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /carbonara-watchdog/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --target x86_64-unknown-linux-musl --bin carbonara-watchdog-tele

FROM alpine:3.19.1 AS runtime
WORKDIR /carbonara-watchdog
RUN addgroup -S cw && adduser -S cw -G cw
COPY --from=builder /carbonara-watchdog/target/x86_64-unknown-linux-musl/release/carbonara-watchdog-tele /usr/local/bin
USER cw

CMD ["/usr/local/bin/carbonara-watchdog-tele"]
