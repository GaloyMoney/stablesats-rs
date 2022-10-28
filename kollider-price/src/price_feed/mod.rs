use futures::{SinkExt, Stream, StreamExt};

use tokio_tungstenite::{connect_async, tungstenite::Message};

mod tick;
pub use tick::*;

pub mod error;
use error::KolliderPriceFeedError;

pub async fn subscribe_price_feed(
) -> Result<std::pin::Pin<Box<dyn Stream<Item = KolliderPriceTicker> + Send>>, KolliderPriceFeedError>
{
    let (ws_stream, _) = connect_async("wss://testnet.kollider.xyz/v1/ws/")
        .await
        .unwrap(); // FIXME

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
