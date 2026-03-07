#!/bin/bash
set -e

echo "🔧 Initializing Zaino Indexer..."

# Configuration
ZEBRA_RPC_HOST=${ZEBRA_RPC_HOST:-zebra}
ZEBRA_RPC_PORT=${ZEBRA_RPC_PORT:-8232}
ZAINO_GRPC_BIND=${ZAINO_GRPC_BIND:-0.0.0.0:9067}
ZAINO_DATA_DIR=${ZAINO_DATA_DIR:-/var/zaino}

# Resolve zebra hostname to IP if needed
echo "🔍 Resolving Zebra hostname..."
RESOLVED_IP=$(getent hosts ${ZEBRA_RPC_HOST} | awk '{ print $1 }' | head -1)
if [ -n "$RESOLVED_IP" ]; then
    echo "Resolved ${ZEBRA_RPC_HOST} to ${RESOLVED_IP}"
    ZEBRA_RPC_HOST=${RESOLVED_IP}
else
    echo "⚠️  Could not resolve ${ZEBRA_RPC_HOST}, using as-is"
fi

echo "Configuration:"
echo "  Zebra RPC:  ${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}"
echo "  gRPC Bind:  ${ZAINO_GRPC_BIND}"
echo "  Data Dir:   ${ZAINO_DATA_DIR}"

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

# Create config directory
mkdir -p ${ZAINO_DATA_DIR}/zainod

# Create Zaino config file with JSONRPC backend
echo "📝 Creating Zaino config file..."
echo "# Zaino Configuration - JSONRPC Backend" > ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "network = \"Regtest\"" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "backend = \"fetch\"" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "[grpc_settings]" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "listen_address = \"${ZAINO_GRPC_BIND}\"" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "insecure = true" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "[validator_settings]" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "validator_jsonrpc_listen_address = \"${ZEBRA_RPC_HOST}:${ZEBRA_RPC_PORT}\"" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "[storage.database]" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml
echo "path = \"${ZAINO_DATA_DIR}\"" >> ${ZAINO_DATA_DIR}/zainod/zindexer.toml

echo "Config created at ${ZAINO_DATA_DIR}/zainod/zindexer.toml"
echo "📄 Config contents:"
cat ${ZAINO_DATA_DIR}/zainod/zindexer.toml

# Change to data dir
cd ${ZAINO_DATA_DIR}

# Start Zaino
echo "Starting Zaino indexer..."
export RUST_BACKTRACE=1
export RUST_LOG=debug
exec zainod