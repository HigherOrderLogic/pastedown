FROM rust:alpine as builder

WORKDIR /app

RUN apt-get update -y && apt-get upgrade -y && apt-get install musl-tools -y

COPY build.rs Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY src src
RUN touch src/main.rs
RUN cargo build --release

FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/target/release/oplud-api .

CMD ./oplud-api