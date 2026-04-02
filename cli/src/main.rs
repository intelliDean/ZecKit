use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;
mod docker;
mod config;
mod error;
mod utils;
pub mod assets;

#[derive(Parser)]
#[command(name = "zeckit")]
#[command(about = "ZecKit - Developer toolkit for Zcash on Zebra", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to the ZecKit project root (overrides auto-detection)
    #[arg(long, global = true)]
    project_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the ZecKit devnet
    Up {
        /// Light-client backend: lwd (lightwalletd) or zaino
        #[arg(short, long, default_value = "lwd")]
        backend: String,
        
        /// Force fresh start (remove volumes)
        #[arg(short, long)]
        fresh: bool,

        /// Startup timeout in minutes
        #[arg(long, default_value = "10")]
        timeout: u64,

        /// Run in action mode (generate artifacts)
        #[arg(long)]
        action_mode: bool,

        /// Custom transparent address for Zebra mining rewards (overrides embedded default)
        #[arg(long)]
        miner_address: Option<String>,

        /// Address to automatically fund after the devnet is healthy
        #[arg(long)]
        fund_address: Option<String>,

        /// Amount in ZEC to send to --fund-address (default: 10.0)
        #[arg(long, default_value = "10.0")]
        fund_amount: f64,
    },
    
    /// Stop the ZecKit devnet
    Down {
        /// Remove volumes (clean slate)
        #[arg(short, long)]
        purge: bool,
    },
    
    /// Show devnet status
    Status,
    
    /// Run smoke tests
    Test {
        /// Amount to send in E2E test
        #[arg(long, default_value = "0.05")]
        amount: f64,

        /// Memo to use for E2E test
        #[arg(long, default_value = "ZecKit E2E Transaction")]
        memo: String,

        /// Run in action mode (generate artifacts)
        #[arg(long)]
        action_mode: bool,
    },

    /// Initialize a GitHub Actions workflow for this project
    #[command(long_about = "Generates a standardized GitHub Actions workflow (.github/workflows/zeckit-e2e.yml) that automatically spins up a 2-node Zebra cluster, configured with your choice of privacy backend and an embedded shielded faucet.")]
    Init {
        /// Light-client backend to use in CI: lwd (lightwalletd) or zaino
        #[arg(short, long, default_value = "zaino", value_parser = ["zaino", "lwd"])]
        backend: String,

        /// Force overwrite of an existing workflow file
        #[arg(short, long)]
        force: bool,

        /// Custom file path for the generated workflow (e.g. .github/workflows/custom.yml)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Up { backend, fresh, timeout, action_mode, miner_address, fund_address, fund_amount } => {
            commands::up::execute(backend, fresh, timeout, action_mode, miner_address, fund_address, fund_amount, cli.project_dir).await
        }
        Commands::Down { purge } => {
            commands::down::execute(purge, cli.project_dir).await
        }
        Commands::Status => {
            commands::status::execute(cli.project_dir).await
        }
        Commands::Test { amount, memo, action_mode } => {
            commands::test::execute(amount, memo, action_mode, cli.project_dir).await
        }
        Commands::Init { backend, force, output } => {
            commands::init::execute(backend, force, output, cli.project_dir).await
        }
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
