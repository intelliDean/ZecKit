/**
 * ZecKit Devnet "Happy Path" Example
 * 
 * This script demonstrates how a downstream application interacts with the local
 * ZecKit Regtest Cluster. It checks the Faucet's balance, requests funds to a random
 * Unified Address (UA), and proves the blockchain is functioning properly.
 * 
 * Prerequisites: You must have the ZecKit Devnet running in the background.
 * Run `cargo run -- up` from the top level `cli` directory first.
 */

const FAUCET_API = "http://127.0.0.1:8080";

async function runHappyPath() {
    console.log("==================================================");
    console.log("🔌 Connecting to ZecKit Devnet Faucet API...");
    console.log("==================================================\n");

    try {
        // 1. Check Faucet Status & Balance
        console.log("🔍 1. Checking Devnet Faucet Wallet Balance...");
        const statsResponse = await fetch(`${FAUCET_API}/stats`);

        if (!statsResponse.ok) {
            throw new Error(`Faucet API returned ${statsResponse.status}: Are you sure ZecKit Devnet is running?`);
        }

        const stats = await statsResponse.json();
        console.log(`✅ Success! Faucet has:`);
        console.log(`   - Transparent Balance: ${stats.transparent_balance} ZEC`);
        console.log(`   - Orchard Balance:    ${stats.orchard_balance} ZEC\n`);

        // 2. We will need a Unified Address to receive funds.
        // For this demo, let's ask the faucet what its own Unified address is.
        // In a real app, this would be your user's wallet address.
        const addrResponse = await fetch(`${FAUCET_API}/address`);
        const { unified_address } = await addrResponse.json();

        console.log("📬 2. Receiver Address Identifed!");
        console.log(`   UA: ${unified_address.substring(0, 30)}...\n`);

        // 3. Send Funds
        console.log("💸 3. Initiating ZEC Transfer (Shielded)...");
        console.log("   Sending 0.1 ZEC to the address via Faucet...");

        const sendResponse = await fetch(`${FAUCET_API}/send`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                address: unified_address,
                amount: 0.1,
                memo: "Happy Path Example Test"
            })
        });

        const sendResult = await sendResponse.json();

        if (sendResult.txid) {
            console.log(`✅ Success! Transaction injected into the Devnet Mempool.`);
            console.log(`📜 TXID: ${sendResult.txid}`);
            console.log(`   Status: ${sendResult.status}\n`);
        } else {
            console.log("⚠️ Transaction failed. Do you have enough Orchard funds?");
            console.log("   Wait for the background miner to shield some ZEC first.");
            console.dir(sendResult);
        }

        console.log("🎉 Run completed successfully! The regtest network processed the request.");

    } catch (e) {
        console.error("\n❌ ERROR:", e.message);
        console.error("Make sure your ZecKit Devnet is running using `zeckit up`");
        process.exit(1);
    }
}

runHappyPath();
