FROM golang:1.24-bookworm as builder

RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN git clone https://github.com/zcash/lightwalletd.git
WORKDIR /build/lightwalletd

# CACHE GO MODULES FIRST
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go mod download

# BUILD WITH PERSISTENT CACHES
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o lightwalletd .

# Build grpc-health-probe
WORKDIR /build
RUN git clone https://github.com/grpc-ecosystem/grpc-health-probe.git
WORKDIR /build/grpc-health-probe
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o grpc_health_probe

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/lightwalletd/lightwalletd /usr/local/bin/lightwalletd
COPY --from=builder /build/grpc-health-probe/grpc_health_probe /usr/local/bin/grpc_health_probe
RUN chmod +x /usr/local/bin/lightwalletd /usr/local/bin/grpc_health_probe

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

RUN mkdir -p /var/lightwalletd

WORKDIR /var/lightwalletd

EXPOSE 9067

ENTRYPOINT ["/entrypoint.sh"]