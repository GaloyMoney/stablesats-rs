use clap::ArgEnum;
use rust_decimal::Decimal;
use tonic::transport::channel::Channel;
use url::Url;

use ::price_server::proto;
type ProtoClient = proto::price_client::PriceClient<Channel>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
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
        let client = ProtoClient::connect(self.config.url.to_string()).await?;
        Ok(client)
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
            _ => unimplemented!(),
        }
        Ok(())
    }
}

fn print_price(direction: Direction, expiry: Option<u64>, original_amount: Decimal, amount: u64) {
    match expiry {
        Some(expiry) => println!(
            "Price to {:?} {} sats within {} seconds - {} cents",
            direction, original_amount, expiry, amount
        ),
        None => println!(
            "Price to {:?} {} sats immediately- {} cents",
            direction, original_amount, amount
        ),
    }
}
