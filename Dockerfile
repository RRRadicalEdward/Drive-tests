FROM rust:latest AS planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:latest AS cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:latest AS builder
WORKDIR app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV OPENSSL_STATIC 1
RUN cargo build --release --bin driving-tests-site

FROM debian:buster-slim AS runtime
WORKDIR app
RUN apt-get update -y \
    && apt-get install --no-install-recommends pkg-config openssl libssl-dev ca-certificates sqlite libsqlite3-dev -y \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/drive_tests_db.db .
COPY --from=builder /app/cert/ .
COPY --from=builder /app/rsa-keys/ .
COPY --from=builder /app/target/release/driving-tests-site .
ENV RUST_LOG DEBUG
ENV SERVER_IP_ADDR 0.0.0.0:5050
ENV DATABASE_URL /app/drive_tests_db.db
ENV CERT_DIR /app/
ENV KEYS_DIR /app/
EXPOSE 5050
ENTRYPOINT ["./driving-tests-site"]
