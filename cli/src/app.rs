use anyhow::Context;
use chrono::Duration;
use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use std::{collections::HashMap, path::PathBuf};
use url::Url;

use super::{config::*, price_client::*};
use shared::pubsub::memory;

#[derive(Parser)]
#[clap(version, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[clap(
        short,
        long,
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
        /// Connection string for the user-trades database
        #[clap(env = "USER_TRADES_PG_CON", default_value = "")]
        user_trades_pg_con: String,
        /// Phone code for the galoy client
        #[clap(env = "GALOY_PHONE_CODE", default_value = "")]
        galoy_phone_code: String,
        /// Connection string for the hedging database
        #[clap(env = "HEDGING_PG_CON", default_value = "")]
        hedging_pg_con: String,
        /// Okex secret key
        #[clap(env = "OKEX_SECRET_KEY", default_value = "")]
        okex_secret_key: String,
        /// Okex passphrase
        #[clap(env = "OKEX_PASSPHRASE", default_value = "")]
        okex_passphrase: String,
    },
    /// Gets a quote from the price server
    Price {
        /// price server URL
        #[clap(short, long, action, value_parser, env = "PRICE_SERVER_URL")]
        url: Option<Url>,
        #[clap(short, long, action, value_enum, value_parser, default_value_t = Direction::Buy)]
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
            user_trades_pg_con,
            galoy_phone_code,
            okex_passphrase,
            okex_secret_key,
            hedging_pg_con,
        } => {
            let config = Config::from_path(
                cli.config,
                EnvOverride {
                    redis_password,
                    user_trades_pg_con,
                    galoy_phone_code,
                    okex_passphrase,
                    okex_secret_key,
                    hedging_pg_con,
                },
            )?;
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
        user_trades,
        tracing,
        galoy,
        hedging,
        kollider_price_feed,
        exchanges,
    }: Config,
) -> anyhow::Result<()> {
    if !exchanges.is_valid() {
        Err(anyhow::anyhow!(
            "Invalid exchanges config - at least one exchange has to be configured"
        ))?;
    }
    println!("Starting server process");
    crate::tracing::init_tracer(tracing)?;
    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = Vec::new();
    let mut checkers = HashMap::new();
    let (price_send, price_recv) = memory::channel(price_stream_throttle_period());

    if okex_price_feed.enabled {
        println!("Starting Okex price feed");

        let okex_send = send.clone();
        let price_send = price_send.clone();
        handles.push(tokio::spawn(async move {
            let _ = okex_send.try_send(
                okex_price::run(okex_price_feed.config, price_send)
                    .await
                    .context("Okex Price Feed error"),
            );
        }));
    }

    if kollider_price_feed.enabled {
        println!("Starting Kollider price feed");

        let kollider_send = send.clone();
        handles.push(tokio::spawn(async move {
            let _ = kollider_send.try_send(
                kollider_price::run(kollider_price_feed.config, price_send)
                    .await
                    .context("Kollider Price Feed error"),
            );
        }));
    }

    if hedging.enabled {
        println!("Starting hedging process");

        let hedging_send = send.clone();
        let pubsub = pubsub.clone();
        let galoy = galoy.clone();
        let (snd, recv) = futures::channel::mpsc::unbounded();
        let price = price_recv.resubscribe();
        checkers.insert("hedging", snd);

        if let Some(okex_cfg) = exchanges.okex.as_ref() {
            let okex_config = okex_cfg.config.clone();
            handles.push(tokio::spawn(async move {
                let _ = hedging_send.try_send(
                    hedging::run(recv, hedging.config, okex_config, galoy, pubsub, price)
                        .await
                        .context("Hedging error"),
                );
            }));
        }
    }

    if price_server.enabled {
        println!(
            "Starting price server on port {}",
            price_server.server.listen_port
        );

        let price_send = send.clone();
        let (snd, recv) = futures::channel::mpsc::unbounded();
        checkers.insert("price", snd);
        let price = price_recv.resubscribe();
        handles.push(tokio::spawn(async move {
            let _ = price_send.try_send(
                price_server::run(
                    recv,
                    price_server.health,
                    price_server.server,
                    price_server.fees,
                    price,
                    price_server.price_cache,
                    exchanges,
                )
                .await
                .context("Price Server error"),
            );
        }));
    }
    if user_trades.enabled {
        println!("Starting user trades process");

        let user_trades_send = send.clone();
        handles.push(tokio::spawn(async move {
            let _ = user_trades_send.try_send(
                user_trades::run(user_trades.config, pubsub, galoy)
                    .await
                    .context("User Trades error"),
            );
        }));
    }
    handles.push(tokio::spawn(async move {
        let _ = send.try_send(crate::health::run(checkers).await);
    }));
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

fn price_stream_throttle_period() -> Duration {
    Duration::from_std(std::time::Duration::from_secs(2)).unwrap()
}
