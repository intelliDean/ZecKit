#!/bin/bash
set -e

# Use provided config file
CONFIG_FILE="/etc/zebrad/zebrad.toml"

if [ -f "$CONFIG_FILE" ]; then
    echo "Starting zebrad with config: $CONFIG_FILE"
    exec zebrad -c "$CONFIG_FILE"
else
    echo "ERROR: Config file not found at $CONFIG_FILE"
    exit 1
fi
