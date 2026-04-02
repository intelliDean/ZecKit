# ========================================
# Builder Stage
# ========================================
FROM rust:1.85-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    build-essential \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Zaino build arguments - can be overridden for different forks/branches
# Default points to the fork with the critical regtest-insecure-grpc fix
ARG ZAINO_REPO=https://github.com/Timi16/zaino.git
ARG ZAINO_BRANCH=fix/regtest-insecure-grpc

WORKDIR /build

# Clone Zaino (Note: ZecKit requires the fix/regtest-insecure-grpc branch 
# for insecure gRPC support in devnet environments)
RUN git clone ${ZAINO_REPO} zaino

WORKDIR /build/zaino

# Checkout specified branch
RUN git checkout ${ZAINO_BRANCH}

# CACHE CARGO DEPENDENCIES FIRST (this is the magic)
ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/zaino/target \
    cargo fetch

# BUILD WITH PERSISTENT CACHES
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/zaino/target \
    cargo build --bin zainod --features no_tls_use_unencrypted_traffic && \
    cp target/debug/zainod /tmp/zainod

# ========================================
# Runtime Stage
# ========================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 2002 -s /bin/bash zaino

COPY --from=builder /tmp/zainod /usr/local/bin/zainod
RUN chmod +x /usr/local/bin/zainod

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

RUN mkdir -p /var/zaino && chown -R zaino:zaino /var/zaino

WORKDIR /var/zaino

USER zaino

ENTRYPOINT ["/entrypoint.sh"]