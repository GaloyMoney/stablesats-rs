use anyhow;
use futures::StreamExt;
use okex_price::{order_book::*, ChannelArgs, PriceFeedConfig};
use url::Url;

#[tokio::test]
async fn subscribe_to_order_book_channel() -> anyhow::Result<()> {
    let config = PriceFeedConfig {
        url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap(),
    };
    let mut order_book_stream = subscribe_btc_usd_swap_order_book(config)
        .await
        .expect("subscribe to order book channel");
    let order_book = order_book_stream.next().await.expect("order book");

    assert_eq!(
        order_book.arg,
        ChannelArgs {
            channel: "books".to_string(),
            inst_id: "BTC-USD-SWAP".to_string(),
        }
    );
    assert_eq!(order_book.data.len(), 1);
    assert_eq!(order_book.data.first().expect("asks").asks.len(), 400);
    assert_eq!(order_book.data.first().expect("bids").bids.len(), 400);
    assert_eq!(order_book.action, OrderBookAction::Snapshot);
    Ok(())
}
