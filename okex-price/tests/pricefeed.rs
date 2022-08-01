use futures::StreamExt;
use okex_price::pricefeed::{subscribe_btc_usd_swap, ChannelArgs, PriceFeedConfig};
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
    let pricetick = received.next().await;

    assert_eq!(pricetick.clone().unwrap().arg, expected_arg);
    assert_eq!(
        type_of(pricetick.clone().unwrap()),
        "okex_price::pricefeed::feeder::OkexPriceTick"
    );
    assert_eq!(
        type_of(pricetick.clone().unwrap().data),
        "alloc::vec::Vec<okex_price::pricefeed::feeder::TickersChannelData>"
    );

    Ok(())
}
