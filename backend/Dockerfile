FROM rust:alpine AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev git
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --no-default-features --features cas --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked --no-default-features --features cas

FROM alpine:latest
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend
ENTRYPOINT ["/app/backend"]
