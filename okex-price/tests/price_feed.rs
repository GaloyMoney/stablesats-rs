use futures::StreamExt;
use okex_price::price_feed::{subscribe_btc_usd_swap, ChannelArgs, PriceFeedConfig};
use std::any::type_name;
use url::Url;

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[tokio::test]
async fn test_subscribe_btc_usd_swap() -> Result<(), anyhow::Error> {
    let expected_arg = ChannelArgs {
        channel: "tickers".to_string(),
        inst_id: "BTC-USD-SWAP".to_string(),
    };

    let config = PriceFeedConfig {
        url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap(),
    };
    let mut received = subscribe_btc_usd_swap(config).await.unwrap();
    let price_tick = received.next().await;

    assert_eq!(price_tick.clone().unwrap().arg, expected_arg);
    assert_eq!(
        type_of(price_tick.clone().unwrap()),
        "okex_price::price_feed::feeder::OkexPriceTick"
    );
    assert_eq!(
        type_of(price_tick.clone().unwrap().data),
        "alloc::vec::Vec<okex_price::price_feed::feeder::TickersChannelData>"
    );

    Ok(())
}
