mod tick;

use futures::{SinkExt, Stream, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub use crate::{config::*, error::*};
pub use tick::*;

pub async fn subscribe_btc_usd_swap_price_tick(
    config: PriceFeedConfig,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = DeribitPriceTick> + Send>>, PriceFeedError> {
    let (ws_stream, _) = connect_async(config.url).await?;
    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = serde_json::json!({
        "method": "public/subscribe",
        "params": {
            "channels": [
                "quote.BTC-PERPETUAL"
            ]
        },
        "jsonrpc": "2.0",
        "id": 0
    })
    .to_string();
    let item = Message::Text(subscribe_args);

    sender.send(item).await?;

    Ok(Box::pin(receiver.filter_map(|message| async {
        if let Ok(msg) = message {
            if let Ok(msg_str) = msg.into_text() {
                if let Ok(tick) = serde_json::from_str::<DeribitPriceTick>(&msg_str) {
                    return Some(tick);
                }
            }
        }
        None
    })))
}
