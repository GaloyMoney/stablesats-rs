use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::config::Config;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {},
}

const DEFAULT_CONFIG: &str = "stablesats.yml";

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG));
    let _config = Config::from_path(config_path)?;

    match cli.command {
        Commands::Run {} => {
            println!("Hello")
        }
    }
    Ok(())
}
