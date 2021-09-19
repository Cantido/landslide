
all:
  BUILD +test

build:
  FROM rust:alpine
  WORKDIR /app

  COPY Cargo.toml .
  COPY src ./src

  RUN cargo build


test:
  FROM +build

  RUN cargo test
