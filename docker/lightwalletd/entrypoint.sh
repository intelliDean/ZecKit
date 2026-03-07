#!/bin/bash
set -e

echo "🔧 Initializing Lightwalletd..."

# Configuration
ZEBRA_RPC_HOST=${ZEBRA_RPC_HOST:-zebra}
ZEBRA_RPC_PORT=${ZEBRA_RPC_PORT:-8232}
LWD_GRPC_BIND=${LWD_GRPC_BIND:-0.0.0.0:9067}

echo "Configuration:"
echo "  Zebra RPC:  ${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}"
echo "  gRPC Bind:  ${LWD_GRPC_BIND}"

# Wait for Zebra
echo " Waiting for Zebra RPC..."
MAX_ATTEMPTS=60
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":"health","method":"getblockcount","params":[]}' \
        "http://${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}" > /dev/null 2>&1; then
        echo "Zebra RPC is ready!"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    sleep 5
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo "❌ Zebra did not become ready in time"
    exit 1
fi

# Get block count
BLOCK_COUNT=$(curl -s \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":"info","method":"getblockcount","params":[]}' \
    "http://${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}" | grep -o '"result":[0-9]*' | cut -d: -f2 || echo "0")

echo "Current block height: ${BLOCK_COUNT}"

# Wait for blocks
echo " Waiting for at least 10 blocks to be mined..."
while [ "${BLOCK_COUNT}" -lt "10" ]; do
    sleep 10
    BLOCK_COUNT=$(curl -s \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","id":"info","method":"getblockcount","params":[]}' \
        "http://${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}" | grep -o '"result":[0-9]*' | cut -d: -f2 || echo "0")
    echo "  Current blocks: ${BLOCK_COUNT}"
done

echo "Zebra has ${BLOCK_COUNT} blocks!"

# Start lightwalletd with RPC credentials (dummy values for Zebra which doesn't require auth)
echo "Starting lightwalletd..."
exec lightwalletd \
    --grpc-bind-addr=${LWD_GRPC_BIND} \
    --data-dir=/var/lightwalletd \
    --log-level=7 \
    --no-tls-very-insecure=true \
    --rpchost=${ZEBRA_RPC_HOST} \
    --rpcport=${ZEBRA_RPC_PORT} \
    --rpcuser=zcash \
    --rpcpassword=zcash