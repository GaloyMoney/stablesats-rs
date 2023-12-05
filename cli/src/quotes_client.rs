use clap::ValueEnum;
use tonic::transport::channel::Channel;
use url::Url;

use ::quotes_server::proto;
type ProtoClient = proto::quote_service_client::QuoteServiceClient<Channel>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum QuoteDirection {
    Buy,
    Sell,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum Currency {
    Sats,
    Cents,
}

pub struct QuotesClientConfig {
    pub url: Url,
}

impl Default for QuotesClientConfig {
    fn default() -> Self {
        Self {
            url: Url::parse("http://localhost:3326").unwrap(),
        }
    }
}

pub struct QuotesClient {
    config: QuotesClientConfig,
}

impl QuotesClient {
    pub fn new(config: QuotesClientConfig) -> Self {
        Self { config }
    }

    async fn connect(&self) -> anyhow::Result<ProtoClient> {
        match ProtoClient::connect(self.config.url.to_string()).await {
            Ok(client) => Ok(client),
            Err(err) => {
                eprintln!(
                    "Couldn't connect to quotes server\nAre you sure its running on {}?\n",
                    self.config.url
                );
                Err(anyhow::anyhow!(err))
            }
        }
    }

    pub async fn get_quote(
        &self,
        direction: QuoteDirection,
        currency: Currency,
        immediate_execution: bool,
        amount: u64,
    ) -> anyhow::Result<()> {
        let mut client = self.connect().await?;

        match (direction, currency) {
            (QuoteDirection::Buy, Currency::Sats) => {
                let request = tonic::Request::new(proto::GetQuoteToSellUsdRequest {
                    quote_for: Some(
                        proto::get_quote_to_sell_usd_request::QuoteFor::AmountToBuyInSats(amount),
                    ),
                    immediate_execution,
                });
                let response = client.get_quote_to_sell_usd(request).await?;
                output_json(response)?;
            }
            (QuoteDirection::Sell, Currency::Sats) => {
                let request = tonic::Request::new(proto::GetQuoteToBuyUsdRequest {
                    quote_for: Some(
                        proto::get_quote_to_buy_usd_request::QuoteFor::AmountToSellInSats(amount),
                    ),
                    immediate_execution,
                });
                let response = client.get_quote_to_buy_usd(request).await?;
                output_json(response)?;
            }
            (QuoteDirection::Buy, Currency::Cents) => {
                let request = tonic::Request::new(proto::GetQuoteToBuyUsdRequest {
                    quote_for: Some(
                        proto::get_quote_to_buy_usd_request::QuoteFor::AmountToBuyInCents(amount),
                    ),
                    immediate_execution,
                });
                let response = client.get_quote_to_buy_usd(request).await?;
                output_json(response)?;
            }
            (QuoteDirection::Sell, Currency::Cents) => {
                let request = tonic::Request::new(proto::GetQuoteToSellUsdRequest {
                    quote_for: Some(
                        proto::get_quote_to_sell_usd_request::QuoteFor::AmountToSellInCents(amount),
                    ),
                    immediate_execution,
                });
                let response = client.get_quote_to_sell_usd(request).await?;
                output_json(response)?;
            }
        };

        Ok(())
    }

    pub async fn accept_quote(&self, quote_id: String) -> anyhow::Result<()> {
        let mut client = self.connect().await?;

        let request = tonic::Request::new(proto::AcceptQuoteRequest { quote_id });
        let _ = client.accept_quote(request).await?.into_inner();
        println!("Quote accepted!");

        Ok(())
    }
}

fn output_json<T: serde::Serialize>(response: tonic::Response<T>) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&response.into_inner())?);
    Ok(())
}
