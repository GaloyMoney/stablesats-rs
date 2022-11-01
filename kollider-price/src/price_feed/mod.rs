use futures::{SinkExt, Stream, StreamExt};

use config::KolliderPriceFeedConfig;
use error::KolliderPriceFeedError;
pub use tick::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub mod config;
pub mod error;
mod tick;

pub async fn subscribe_price_feed(
    config: KolliderPriceFeedConfig,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = KolliderPriceTicker> + Send>>, KolliderPriceFeedError>
{
    let (ws_stream, _) = connect_async(config.url).await?;

    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = serde_json::json!({
        "type": "subscribe",
        "symbols": ["BTCUSD.PERP"],
        "channels": ["ticker"]
    })
    .to_string();
    let item = Message::Text(subscribe_args);

    sender.send(item).await.unwrap();

    Ok(Box::pin(receiver.filter_map(|msg| async {
        let msg = msg.unwrap();
        if let Message::Text(txt) = msg {
            println!("tick raw: {}", txt);

            if !txt.contains("success") {
                let ticker: KolliderPriceTickerRoot = serde_json::from_str(&txt).unwrap();
                println!("tick: {:?}", ticker.data);
                return Some(ticker.data);
            }
        }
        None
    })))
}
