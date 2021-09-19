# SPDX-FileCopyrightText: 2021 Rosa Richter
#
# SPDX-License-Identifier: MIT

all:
  BUILD +test
  BUILD +lint
  BUILD +lint-formatting
  BUILD +lint-copyright

build:
  FROM rust
  WORKDIR /app

  COPY Cargo.toml .
  COPY src ./src

  RUN cargo build


test:
  FROM +build

  RUN cargo test

lint:
  FROM +build

  RUN rustup component add clippy

  RUN cargo clippy

lint-formatting:
  FROM +build

  RUN rustup component add rustfmt

  RUN cargo fmt -- --check

lint-copyright:
  FROM fsfe/reuse

  COPY . .

  RUN reuse lint
