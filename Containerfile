# SPDX-License-Identifier: MPL-2.0 OR Apache-2.0
# SPDX-FileCopyrightText: 2025 hyperpolymath

FROM rust:1.85-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src/ src/

RUN cargo build --release --bin conflow

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/conflow /usr/local/bin/conflow

ENTRYPOINT ["conflow"]
CMD ["--help"]
