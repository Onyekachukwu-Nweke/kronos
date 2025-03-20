use clap::{Parser, Subcommand};
use commands::backup::run_backup;
use config::Config;
use error::Result;
use logger::init_logger;
use log::info;

mod config;
mod error;
mod logger;
mod commands;
mod utils;
mod storage;
mod backup;
mod database;

#[derive(Parser)]
#[clap(name = "kroncli", about = "A database backup utility")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform a single backup
    Backup {
        #[clap(long, default_value = "config.toml")]
        config: String,
    },
    /// Start the scheduler for automatic backups (Incoming Features)
    /// Restore from a backup (Incoming Features)
}

#[tokio::main]
async fn main() {
    init_logger();
    info!("Starting kroncli");
    let cli = Cli::parse();

    match cli.command {
        Commands::Backup { config } => {
            let cfg = Config::load(&config)?;
        }
    }
}
