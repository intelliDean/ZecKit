#!/bin/bash
set -e

echo "🔧 Initializing Zingo Wallet..."

# Get backend URI from environment variable (set by docker-compose)
BACKEND_URI=${LIGHTWALLETD_URI:-http://lightwalletd:9067}

# Extract hostname from URI for health check
BACKEND_HOST=$(echo $BACKEND_URI | sed 's|http://||' | cut -d: -f1)
BACKEND_PORT=$(echo $BACKEND_URI | sed 's|http://||' | cut -d: -f2)

echo "Configuration:"
echo "  Backend URI:  ${BACKEND_URI}"
echo "  Backend Host: ${BACKEND_HOST}"
echo "  Backend Port: ${BACKEND_PORT}"

# Wait for backend (lightwalletd OR zaino)
echo " Waiting for backend (${BACKEND_HOST})..."
MAX_ATTEMPTS=60
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if nc -z ${BACKEND_HOST} ${BACKEND_PORT} 2>/dev/null; then
        echo "Backend port is open!"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    echo "Attempt $ATTEMPT/$MAX_ATTEMPTS - backend not ready yet..."
    sleep 2
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo "❌ Backend did not become ready in time"
    exit 1
fi

# Give backend time to initialize
echo " Giving backend 30 seconds to fully initialize..."
sleep 30

# Create wallet if doesn't exist
if [ ! -f "/var/zingo/zingo-wallet.dat" ]; then
    echo "📝 Creating new wallet..."
    
    # Initialize wallet with --chain regtest
    zingo-cli --data-dir /var/zingo \
              --server ${BACKEND_URI} \
              --chain regtest \
              --nosync << 'EOF'
quit
EOF
    
    echo "Wallet created!"
    
    # Get wallet's unified address
    WALLET_ADDRESS=$(zingo-cli --data-dir /var/zingo \
                               --server ${BACKEND_URI} \
                               --chain regtest \
                               --nosync << 'EOF' | grep '"encoded_address"' | grep -o 'uregtest[a-z0-9]*' | head -1
addresses
quit
EOF
)
    
    echo "📍 Wallet UA: $WALLET_ADDRESS"
    echo "$WALLET_ADDRESS" > /var/zingo/faucet-address.txt
    
    # Generate transparent address for mining
    echo "🔑 Generating transparent address for mining..."
    zingo-cli --data-dir /var/zingo \
              --server ${BACKEND_URI} \
              --chain regtest \
              --nosync << 'EOF' || true
new_taddress_allow_gap
quit
EOF
    
    # Get transparent address
    T_ADDR=$(zingo-cli --data-dir /var/zingo \
                       --server ${BACKEND_URI} \
                       --chain regtest \
                       --nosync << 'EOF' | grep '"encoded_address"' | grep -o 'tm[a-zA-Z0-9]*' | head -1
t_addresses
quit
EOF
)
    
    if [ -n "$T_ADDR" ]; then
        echo "📍 Transparent Address: $T_ADDR"
        echo "$T_ADDR" > /var/zingo/mining-address.txt
        
        # Update Zebra config with this address
        # Note: This requires the zebra config to be mounted or accessible
        echo "⚠️  IMPORTANT: Set Zebra miner_address to: $T_ADDR"
        echo "   Add this to docker/configs/zebra.toml:"
        echo "   miner_address = \"$T_ADDR\""
    else
        echo "⚠️  Could not get transparent address"
    fi
else
    echo "Existing wallet found"
    
    # Get existing addresses
    WALLET_ADDRESS=$(zingo-cli --data-dir /var/zingo \
                               --server ${BACKEND_URI} \
                               --chain regtest \
                               --nosync << 'EOF' | grep '"encoded_address"' | grep -o 'uregtest[a-z0-9]*' | head -1
addresses
quit
EOF
)
    echo "📍 Wallet UA: $WALLET_ADDRESS"
    
    T_ADDR=$(zingo-cli --data-dir /var/zingo \
                       --server ${BACKEND_URI} \
                       --chain regtest \
                       --nosync << 'EOF' | grep '"encoded_address"' | grep -o 'tm[a-zA-Z0-9]*' | head -1
t_addresses
quit
EOF
)
    
    if [ -n "$T_ADDR" ]; then
        echo "📍 Transparent Address: $T_ADDR"
    fi
fi

# Sync wallet (ignore errors if no blocks yet)
echo " Syncing wallet (will complete after blocks are mined)..."
zingo-cli --data-dir /var/zingo \
          --server ${BACKEND_URI} \
          --chain regtest << 'EOF' || true
sync run
quit
EOF

echo "Wallet is ready! (Sync will complete after mining blocks)"

# Keep container running
tail -f /dev/null