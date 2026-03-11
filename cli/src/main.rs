use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;
mod docker;
mod config;
mod error;
mod utils;

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
        #[arg(short, long, default_value = "none")]
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Up { backend, fresh, timeout, action_mode } => {
            commands::up::execute(backend, fresh, timeout, action_mode, cli.project_dir).await
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
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}