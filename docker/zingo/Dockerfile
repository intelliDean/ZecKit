# syntax=docker/dockerfile:1.7

########################################
# 1️⃣ Builder Stage
########################################
FROM rust:1.80-slim-bookworm as builder

ARG ZINGOLIB_VERSION=zingolib_v3.0.1

RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    protobuf-compiler \
    libssl-dev \
    libsqlite3-dev \
    curl \
    git \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Clone pinned version for reproducibility
RUN git clone --depth 1 --branch ${ZINGOLIB_VERSION} \
    https://github.com/zingolabs/zingolib.git

WORKDIR /build/zingolib

# CACHE DEPENDENCIES FIRST
ENV CARGO_HOME=/usr/local/cargo

########################################
# 2️⃣ Dependency Cache Layer
########################################
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch

########################################
# 3️⃣ Build Layer (Release Mode)
########################################
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/zingolib/target \
    cargo build --release \
    --package zingo-cli \
    --locked && \
    cp target/release/zingo-cli /usr/local/bin/ && \
    chmod +x /usr/local/bin/zingo-cli

########################################
# 4️⃣ Runtime Stage
########################################
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    netcat-openbsd \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/zingo-cli /usr/local/bin/zingo-cli

RUN mkdir -p /var/zingo && chmod 777 /var/zingo

WORKDIR /var/zingo

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]