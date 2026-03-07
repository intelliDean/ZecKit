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
    Test,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Up { backend, fresh } => {
            commands::up::execute(backend, fresh).await
        }
        Commands::Down { purge } => {
            commands::down::execute(purge).await
        }
        Commands::Status => {
            commands::status::execute().await
        }
        Commands::Test => {
            commands::test::execute().await
        }
    };
    
    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}