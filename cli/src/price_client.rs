use clap::ValueEnum;
use rust_decimal::Decimal;
use tonic::transport::channel::Channel;
use url::Url;

use ::price_server::proto;
type ProtoClient = proto::price_service_client::PriceServiceClient<Channel>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Direction {
    Buy,
    Sell,
}

pub struct PriceClientConfig {
    pub url: Url,
}
impl Default for PriceClientConfig {
    fn default() -> Self {
        Self {
            url: Url::parse("http://localhost:3325").unwrap(),
        }
    }
}

pub struct PriceClient {
    config: PriceClientConfig,
}
impl PriceClient {
    pub fn new(config: PriceClientConfig) -> Self {
        Self { config }
    }

    async fn connect(&self) -> anyhow::Result<ProtoClient> {
        match ProtoClient::connect(self.config.url.to_string()).await {
            Ok(client) => Ok(client),
            Err(err) => {
                eprintln!(
                    "Couldn't connect to price server\nAre you sure its running on {}?\n",
                    self.config.url
                );
                Err(anyhow::anyhow!(err))
            }
        }
    }

    pub async fn get_price(
        &self,
        direction: Direction,
        expiry: Option<u64>,
        amount: Decimal,
    ) -> anyhow::Result<()> {
        let mut client = self.connect().await?;

        match (direction, expiry) {
            (Direction::Buy, None) => {
                let request = tonic::Request::new(proto::GetCentsFromSatsForImmediateBuyRequest {
                    amount_in_satoshis: amount.try_into()?,
                });
                let response = client
                    .get_cents_from_sats_for_immediate_buy(request)
                    .await?;
                print_price(
                    direction,
                    None,
                    amount,
                    response.into_inner().amount_in_cents,
                );
            }
            (Direction::Sell, None) => {
                let request = tonic::Request::new(proto::GetCentsFromSatsForImmediateSellRequest {
                    amount_in_satoshis: amount.try_into()?,
                });
                let response = client
                    .get_cents_from_sats_for_immediate_sell(request)
                    .await?;
                print_price(
                    direction,
                    None,
                    amount,
                    response.into_inner().amount_in_cents,
                );
            }
            (Direction::Buy, Some(time_in_seconds)) => {
                let request = tonic::Request::new(proto::GetCentsFromSatsForFutureBuyRequest {
                    amount_in_satoshis: amount.try_into()?,
                    time_in_seconds,
                });
                let response = client.get_cents_from_sats_for_future_buy(request).await?;
                print_price(
                    direction,
                    expiry,
                    amount,
                    response.into_inner().amount_in_cents,
                );
            }
            (Direction::Sell, Some(time_in_seconds)) => {
                let request = tonic::Request::new(proto::GetCentsFromSatsForFutureSellRequest {
                    amount_in_satoshis: amount.try_into()?,
                    time_in_seconds,
                });
                let response = client.get_cents_from_sats_for_future_sell(request).await?;
                print_price(
                    direction,
                    expiry,
                    amount,
                    response.into_inner().amount_in_cents,
                );
            }
        }
        Ok(())
    }
}

fn print_price(direction: Direction, expiry: Option<u64>, original_amount: Decimal, amount: u64) {
    match expiry {
        Some(expiry) => println!(
            "Price to {direction:?} {original_amount} sats within {expiry} seconds - {amount} cents",
        ),
        None => {
            println!("Price to {direction:?} {original_amount} sats immediately - {amount} cents",)
        }
    }
}
