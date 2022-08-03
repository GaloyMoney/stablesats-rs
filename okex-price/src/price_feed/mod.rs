mod config;
mod error;
mod feeder;

pub use config::*;
pub use feeder::*;

use futures::{SinkExt, Stream, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub use error::PriceFeedError;

pub async fn subscribe_btc_usd_swap(
    config: PriceFeedConfig,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = OkexPriceTick> + Send>>, PriceFeedError> {
    let (ws_stream, _) = connect_async(config.url).await?;
    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = serde_json::json!({
        "op": "subscribe",
        "args": [
           {
                "channel": "tickers",
                "instId": "BTC-USD-SWAP"
            }
        ]
    })
    .to_string();
    let item = Message::Text(subscribe_args);

    sender.send(item).await?;

    Ok(Box::pin(receiver.filter_map(|message| async {
        if let Ok(msg) = message {
            if let Ok(msg_str) = msg.into_text() {
                if let Ok(tick) = serde_json::from_str::<OkexPriceTick>(&msg_str) {
                    return Some(tick);
                }
            }
        }
        None
    })))
}
