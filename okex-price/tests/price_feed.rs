use chrono::Duration;
use futures::StreamExt;
use okex_price::price_feed::{subscribe_btc_usd_swap, ChannelArgs, PriceFeedConfig};
use url::Url;

use shared::time::*;

#[tokio::test]
async fn test_subscribe_btc_usd_swap() -> Result<(), anyhow::Error> {
    let config = PriceFeedConfig {
        url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap(),
    };
    let mut received = subscribe_btc_usd_swap(config)
        .await
        .expect("subscribe_btc_usd_swap");
    let price_tick = received.next().await.expect("expected price tick");

    assert_eq!(
        price_tick.clone().arg,
        ChannelArgs {
            channel: "tickers".to_string(),
            inst_id: "BTC-USD-SWAP".to_string(),
        }
    );
    assert_eq!(price_tick.data.len(), 1);
    assert!(
        TimeStamp::now()
            - TimeStamp::try_from(&price_tick.data[0].ts).expect("couldn't convert timestamp")
            < Duration::seconds(5)
    );

    Ok(())
}
