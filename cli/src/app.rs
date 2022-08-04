use anyhow::Context;
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
    let config = Config::from_path(config_path)?;

    match cli.command {
        Commands::Run {} => run_cmd(config).await?,
    }
    Ok(())
}

async fn run_cmd(
    Config {
        pubsub,
        price_server,
        okex_price_feed,
    }: Config,
) -> anyhow::Result<()> {
    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = Vec::new();

    if price_server.enabled {
        let price_send = send.clone();
        let pubsub = pubsub.clone();
        handles.push(tokio::spawn(async move {
            let _ = price_send.try_send(
                price_server::run(price_server.config, pubsub)
                    .await
                    .context("Price Server error"),
            );
        }));
    }
    if okex_price_feed.enabled {
        let okex_send = send.clone();
        handles.push(tokio::spawn(async move {
            let _ = okex_send.try_send(
                okex_price::run(okex_price_feed.config, pubsub)
                    .await
                    .context("Okex Price Feed error"),
            );
        }));
    }
    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        handle.abort();
    }
    reason
}
