#!/bin/bash
set -e

# Use provided config file
CONFIG_FILE="/etc/zebrad/zebrad.toml"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Zebra Entrypoint starting..."
echo "  Config: $CONFIG_FILE"
echo "  User: $(whoami)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$CONFIG_FILE" ]; then
    echo "✓ Config file found"
    echo "  Config size: $(wc -c < "$CONFIG_FILE") bytes"
    zebrad --version
    
    # Check if this is the sync node and wait for miner if so
    if grep -q "zebra-miner:8233" "$CONFIG_FILE"; then
      echo "Sync node detected. Waiting for miner (zebra-miner:8233)..."
      # Try for 60 seconds
      UNTIL=$((SECONDS + 60))
      while [ $SECONDS -lt $UNTIL ]; do
        if curl -s --connect-timeout 2 zebra-miner:8233 >/dev/null 2>&1; then
          echo "✓ Miner found!"
          break
        fi
        echo "  ...still waiting for miner..."
        sleep 5
      done
    fi

    echo "Starting zebrad..."
    exec zebrad -c "$CONFIG_FILE" start
else
    echo "ERROR: Config file not found at $CONFIG_FILE"
    ls -R /etc/zebrad
    exit 1
fi
