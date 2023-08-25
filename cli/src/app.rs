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
        /// Output config on crash
        #[clap(env = "CRASH_REPORT_CONFIG")]
        crash_report_config: Option<bool>,
        /// Connection string for the stablesats database
        #[clap(env = "PG_CON", default_value = "")]
        pg_con: String,
        /// Phone code for the galoy client
        #[clap(env = "GALOY_PHONE_CODE", default_value = "")]
        galoy_phone_code: String,
        /// Okex secret key
        #[clap(env = "OKEX_SECRET_KEY", default_value = "")]
        okex_secret_key: String,
        /// Okex passphrase
        #[clap(env = "OKEX_PASSPHRASE", default_value = "")]
        okex_passphrase: String,
        /// Bitfinex secret key
        #[clap(env = "BITFINEX_SECRET_KEY", default_value = "")]
        bitfinex_secret_key: String,
        /// Bria url
        #[clap(env = "BRIA_URL", default_value = "")]
        bria_url: String,
        /// Bria key
        #[clap(env = "BRIA_KEY", default_value = "")]
        bria_key: String,
        /// Bria wallet name
        #[clap(env = "BRIA_WALLET_NAME", default_value = "")]
        bria_wallet_name: String,
        /// Bria payout queue name
        #[clap(env = "BRIA_PAYOUT_QUEUE_NAME", default_value = "")]
        bria_payout_queue_name: String,
        /// Bria address external id
        #[clap(env = "BRIA_EXTERNAL_ID", default_value = "")]
        bria_external_id: String,
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
            crash_report_config,
            galoy_phone_code,
            okex_passphrase,
            okex_secret_key,
            bitfinex_secret_key,
            pg_con,
            bria_url,
            bria_key,
            bria_wallet_name,
            bria_payout_queue_name,
            bria_external_id,
        } => {
            let config = Config::from_path(
                cli.config,
                EnvOverride {
                    galoy_phone_code,
                    okex_passphrase,
                    okex_secret_key,
                    pg_con,
                    bitfinex_secret_key,
                    bria_url,
                    bria_key,
                    bria_wallet_name,
                    bria_payout_queue_name,
                    bria_external_id,
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
        db,
        price_server,
        bitfinex_price_feed,
        user_trades,
        tracing,
        galoy,
        hedging,
        exchanges,
        bria,
    }: Config,
) -> anyhow::Result<()> {
    println!("Stablesats - v{}", env!("CARGO_PKG_VERSION"));
    println!("Starting server process");
    crate::tracing::init_tracer(tracing)?;

    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = Vec::new();
    let mut checkers = HashMap::new();
    let (price_send, price_recv) = memory::channel(price_stream_throttle_period());

    if exchanges
        .okex
        .as_ref()
        .map(|okex| okex.weight > Decimal::ZERO)
        .unwrap_or(false)
    {
        println!("Starting Okex price feed");

        let okex_send = send.clone();
        let price_send = price_send.clone();
        handles.push(tokio::spawn(async move {
            let _ = okex_send.try_send(
                okex_price::run(price_send)
                    .await
                    .context("Okex Price Feed error"),
            );
        }));
    }

    if bitfinex_price_feed.enabled {
        println!("Starting Bitfinex price feed");

        let bitfinex_send = send.clone();
        let price_send = price_send.clone();
        handles.push(tokio::spawn(async move {
            let _ = bitfinex_send.try_send(
                bitfinex_price::run(bitfinex_price_feed.config, price_send)
                    .await
                    .context("Bitfinex Price Feed error"),
            );
        }));
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
        let weights = extract_weights(&exchanges);
        handles.push(tokio::spawn(async move {
            let _ = price_send.try_send(
                price_server::run(
                    recv,
                    price_server.health,
                    price_server.server,
                    price_server.fees,
                    price,
                    price_server.price_cache,
                    weights,
                )
                .await
                .context("Price Server error"),
            );
        }));
    }

    let mut pool = None;

    if hedging.enabled {
        println!("Starting hedging process");

        let hedging_send = send.clone();
        let galoy = galoy.clone();
        let bria = bria.clone();
        let (snd, recv) = futures::channel::mpsc::unbounded();
        let price = price_recv.resubscribe();
        checkers.insert("hedging", snd);

        if let Some(okex_cfg) = exchanges.okex.as_ref() {
            let okex_config = okex_cfg.config.clone();
            pool = Some(crate::db::init_pool(&db).await?);
            let pool = pool.as_ref().unwrap().clone();
            handles.push(tokio::spawn(async move {
                let _ = hedging_send.try_send(
                    hedging::run(pool, recv, hedging.config, okex_config, galoy, bria, price)
                        .await
                        .context("Hedging error"),
                );
            }));
        }
    }

    if user_trades.enabled {
        println!("Starting user trades process");

        let user_trades_send = send.clone();
        let pool = if let Some(pool) = pool {
            pool
        } else {
            crate::db::init_pool(&db).await?
        };
        handles.push(tokio::spawn(async move {
            let _ = user_trades_send.try_send(
                user_trades::run(pool, user_trades.config, galoy)
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

fn extract_weights(config: &hedging::ExchangesConfig) -> price_server::ExchangeWeights {
    price_server::ExchangeWeights {
        okex: config.okex.as_ref().map(|c| c.weight),
        bitfinex: None,
    }
}
