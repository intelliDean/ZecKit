# zeckit CLI

Command-line tool for managing ZecKit development environment.

## Installation

### From Source

```bash
cd cli
cargo build --release
```

The binary will be at `target/release/zeckit` (or `zeckit.exe` on Windows).

### Add to PATH

**Linux/macOS:**

```bash
sudo cp target/release/zeckit /usr/local/bin/
```

**Windows (PowerShell as Admin):**

```powershell
copy target\release\zeckit.exe C:\Windows\System32\
```

## Usage

### Start Devnet

```bash
# Start Zebra + Faucet only
zeckit up

# Start with lightwalletd
zeckit up --backend lwd

# Start with Zaino (experimental)
zeckit up --backend zaino

# Fresh start (remove old data)
zeckit up --fresh
```

### Stop Devnet

```bash
# Stop services (keep data)
zeckit down

# Stop and remove volumes
zeckit down --purge
```

### Check Status

```bash
zeckit status
```

### Run Tests

```bash
zeckit test
```

## Commands

| Command  | Description         |
| -------- | ------------------- |
| `up`     | Start the devnet    |
| `down`   | Stop the devnet     |
| `status` | Show service status |
| `test`   | Run smoke tests     |

## Options

### `zeckit up`

- `--backend <BACKEND>` - Backend to use: `lwd` (lightwalletd) or `zaino`
- `--fresh` - Remove old data and start fresh

### `zeckit down`

- `--purge` - Remove volumes (clean slate)

## Examples

```bash
# Start everything
zeckit up --backend lwd

# Check if running
zeckit status

# Run tests
zeckit test

# Stop and clean up
zeckit down --purge
```

## Development

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- up
cargo run -- status
cargo run -- test
cargo run -- down
```

### Test

```bash
cargo test
```

## Troubleshooting

### Docker not found

```bash
# Install Docker: https://docs.docker.com/get-docker/
```

### Services not starting

```bash
# Check Docker is running
docker ps

# View logs
docker compose logs zebra
docker compose logs faucet
```

### Port conflicts

```bash
# Stop other services using:
# - 8232 (Zebra RPC)
# - 8080 (Faucet API)
# - 9067 (Backend)
```

## License

MIT OR Apache-2.0
