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

    let subscribe_args = Subscribe::new();
    let serialized_args = serde_json::to_string(&subscribe_args)?;
    let item = Message::Text(serialized_args);

    let subscribe = sender.send(item);
    subscribe.await?;

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
