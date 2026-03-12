# ZecKit Example App

This folder demonstrates exactly how a standard downstream developer application natively connects to the ZecKit Devnet.

Many users assume ZecKit is purely a GitHub Action script. In reality, ZecKit orchestrates a local, completely self-contained **Zcash Regtest Node Cluster** running in the background.

This trivial Node.js script proves that the devnet exposes standard Web3 RPC and Faucet APIs that any standard wallet or dApp can interact with via `fetch` HTTP calls.

## How to Run It

### 1. Start the ZecKit Devnet (The Regtest Cluster)
Open a terminal and start the devnet from the root of the ZecKit repo.
*(If you have compiled the CLI, you can use `./cli/target/debug/zeckit up`. If not, use `cargo`)*:

```bash
cd ZecKit/cli
cargo run -- up --backend zaino
```

Wait for the golden summary showing that all services are fully ready. Be aware that the devnet will take a minute or two to mine enough blocks to shield ZEC into Orchard.

### 2. Run the Sample Application
Open a **second** terminal window, navigate into this `example-app` directory, and run the script:

```bash
cd ZecKit/example-app
npm start
```

### What You Will See
The script will ping the `http://127.0.0.1:8080/stats` endpoint to verify the Regtest Node is running, fetch a wallet address, and instantly submit a 0.1 ZEC Shielded transaction (`POST /send`), returning a fully valid transaction hash.
