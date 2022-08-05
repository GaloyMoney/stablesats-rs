use anyhow::Context;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::path::PathBuf;

use super::config::Config;
use super::price_client::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[clap(
        short,
        long,
        parse(from_os_str),
        env = "STABLESATS_CONFIG",
        default_value = "stablesats.yml",
        value_name = "FILE"
    )]
    config: PathBuf,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Runs the configured processes
    Run,
    /// Gets a quote from the price server
    Price {
        #[clap(short, long, action, arg_enum, value_parser, default_value_t = Direction::Buy)]
        direction: Direction,
        /// For option price expiry in seconds
        #[clap(short, long, action)]
        expiry: Option<u64>,
        amount: Decimal,
    },
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run => {
            let config = Config::from_path(cli.config)?;
            run_cmd(config).await?
        }
        Command::Price {
            direction,
            expiry,
            amount,
        } => price_cmd(direction, expiry, amount).await?,
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
    println!("Starting server process");
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

async fn price_cmd(
    direction: Direction,
    expiry: Option<u64>,
    amount: Decimal,
) -> anyhow::Result<()> {
    let client = PriceClient::new(PriceClientConfig::default());
    client.get_price(direction, expiry, amount).await
}
