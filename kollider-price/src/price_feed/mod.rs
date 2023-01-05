use futures::{SinkExt, Stream, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::PriceFeedError;
use config::KolliderPriceFeedConfig;
pub use tick::*;

pub mod config;
mod tick;

pub async fn subscribe_price_feed(
    config: KolliderPriceFeedConfig,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = KolliderPriceTicker> + Send>>, PriceFeedError> {
    let (ws_stream, _) = connect_async(config.url).await?;

    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = serde_json::json!({
        "type": "subscribe",
        "symbols": ["BTCUSD.PERP"],
        "channels": ["ticker"]
    })
    .to_string();
    let item = Message::Text(subscribe_args);

    sender.send(item).await?;

    Ok(Box::pin(receiver.filter_map(|msg| async {
        if let Ok(msg) = msg {
            if let Ok(txt) = msg.into_text() {
                if !txt.contains("success") {
                    if let Ok(ticker) = serde_json::from_str::<KolliderPriceTickerRoot>(&txt) {
                        return Some(ticker.data);
                    }
                }
            }
        }
        None
    })))
}
