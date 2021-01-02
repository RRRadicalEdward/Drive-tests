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
RUN cargo target add x86_64-unknown-linux-musl
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin driving-tests-site

FROM debian:buster-slim AS runtime
WORKDIR app
RUN apt-get update -y \
    && apt-get install --no-install-recommends pkg-config openssl libssl-dev ca-certificates apt-utils sqlite libsqlite3-dev -y \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
ENV OPENSSL_STATIC 1
ENV RUST_LOG TRACE
ENV SERVER_IP_ADDR 0.0.0.0:5050
ENV CERT_DIR /app/cert/
EXPOSE 5050
ENTRYPOINT ["./app/target/x86_64-unknown-linux-musl/release/driving-tests-site"]
