use futures::{SinkExt, StreamExt};

use tokio_tungstenite::{connect_async, tungstenite::Message};

mod tick;
pub use tick::*;

pub async fn poll_price() {
    let (ws_stream, _) = connect_async("wss://testnet.kollider.xyz/v1/ws/")
        .await
        .unwrap();

    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = serde_json::json!({
        "type": "subscribe",
        "symbols": ["BTCUSD.PERP"],
        "channels": ["ticker"]
    })
    .to_string();
    let item = Message::Text(subscribe_args);

    sender.send(item).await.unwrap();

    receiver
        .for_each(|message| async {
            let msg = message.unwrap();

            if let Message::Text(txt) = msg {
                println!("tick raw: {}", txt);

                // connected msg:
                // {"data":"Subscribed to channel \"ticker:BTCUSD.PERP\" successfully.","type":"success"}

                // ticker msg:
                // {"data":{"best_ask":"20738.0","best_bid":"20733.0","last_price":"20738.5","last_quantity":14,"last_side":"Bid","mid":"20735.5","symbol":"BTCUSD.PERP"},"seq":1,"type":"ticker"}

                if !txt.contains("success") {
                    let ticker: KolliderPriceTickerRoot = serde_json::from_str(&txt).unwrap();
                    println!("tick: {:?}", ticker.data);
                } else {
                    println!("connect: {:?}", txt);
                }
            }
        })
        .await;
}
