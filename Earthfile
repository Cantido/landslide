
all:
  BUILD +test

build:
  FROM rust:alpine
  WORKDIR /app

  COPY Cargo.toml .
  COPY Cargo.lock .
  COPY src ./src

  RUN cargo build


test:
  FROM +build

  RUN cargo test
