# syntax=docker/dockerfile:1.7

########################################
# 1️⃣ Builder Stage
########################################
FROM rust:1.80-bookworm as builder

ARG ZEBRA_VERSION=main

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    libclang-dev \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Clone pinned version for reproducibility
RUN git clone --depth 1 --branch ${ZEBRA_VERSION} \
    https://github.com/ZcashFoundation/zebra.git

WORKDIR /build/zebra

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
    --mount=type=cache,target=/build/zebra/target \
    cargo build --release \
    --features internal-miner \
    --bin zebrad \
    --locked && \
    cp target/release/zebrad /tmp/zebrad

########################################
# 4️⃣ Runtime Stage
########################################
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/zebrad /usr/local/bin/zebrad

RUN mkdir -p /var/zebra/state /root/.cache/zebra

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /var/zebra

EXPOSE 8232 8233

ENTRYPOINT ["/entrypoint.sh"]
CMD ["zebrad", "start"]