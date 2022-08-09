use anyhow::Context;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::path::PathBuf;
use url::Url;

use super::config::*;
use super::price_client::*;

#[derive(Parser)]
#[clap(version, long_about = None)]
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
    Run {
        /// Optional env var for redis password
        #[clap(env = "REDIS_PASSWORD")]
        redis_password: Option<String>,
        /// Output config on crash
        #[clap(env = "CRASH_REPORT_CONFIG")]
        crash_report_config: Option<bool>,
    },
    /// Gets a quote from the price server
    Price {
        /// price server URL
        #[clap(short, long, action, value_parser, env = "PRICE_SERVER_URL")]
        url: Option<Url>,
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
        Command::Run {
            redis_password,
            crash_report_config,
        } => {
            let config = Config::from_path(cli.config, EnvOverride { redis_password })?;
            match (run_cmd(config.clone()).await, crash_report_config) {
                (Err(e), Some(true)) => {
                    println!("Stablesats was started with the following config:");
                    println!("{}", serde_yaml::to_string(&config).unwrap());
                    return Err(e);
                }
                (Err(e), _) => return Err(e),
                _ => (),
            }
        }
        Command::Price {
            url,
            direction,
            expiry,
            amount,
        } => price_cmd(url, direction, expiry, amount).await?,
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
                price_server::run(price_server.server, price_server.fees, pubsub)
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
    url: Option<Url>,
    direction: Direction,
    expiry: Option<u64>,
    amount: Decimal,
) -> anyhow::Result<()> {
    let client = PriceClient::new(
        url.map(|url| PriceClientConfig { url })
            .unwrap_or_else(PriceClientConfig::default),
    );
    client.get_price(direction, expiry, amount).await
}
