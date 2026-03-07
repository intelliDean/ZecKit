# ========================================
# Builder Stage
# ========================================
FROM rust:1.92-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    protobuf-compiler \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy everything
COPY . .

# Build release
RUN cargo build --release

# ========================================
# Runtime Stage
# ========================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 2001 -s /bin/bash faucet

COPY --from=builder /build/target/release/zeckit-faucet /usr/local/bin/faucet
RUN chmod +x /usr/local/bin/faucet

RUN mkdir -p /var/zingo && chown -R faucet:faucet /var/zingo

WORKDIR /var/zingo
USER faucet

EXPOSE 8080

ENV RUST_LOG=info
ENV ZINGO_DATA_DIR=/var/zingo

HEALTHCHECK --interval=30s --timeout=10s --retries=3 --start-period=90s \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["faucet"]