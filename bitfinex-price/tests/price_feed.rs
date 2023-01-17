use bitfinex_price::config::PriceFeedConfig;
use url::Url;

use bitfinex_price::*;
use futures::StreamExt;
use shared::{payload::*, pubsub::*};

#[tokio::test]
async fn subscribes_to_tickers_channel() -> anyhow::Result<()> {
    let config = PriceFeedConfig::default();
    let mut received = subscribe_btc_usd_swap_price_tick(config)
        .await
        .expect("subscribe_btc_usd_swap");
    let price_tick = received.next().await.expect("expected price tick");

    assert!(price_tick.tick.ask >= price_tick.tick.bid);
    Ok(())
}

#[tokio::test]
async fn publishes_to_price_stream() -> anyhow::Result<()> {
    let (tick_send, mut tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());

    let _ = tokio::spawn(async move {
        let config = PriceFeedConfig {
            url: Url::parse("wss://testnet.bitfinex.xyz/v1/ws/").unwrap(),
        };
        let _ = bitfinex_price::run(config, tick_send).await;
    });

    let received_tick = tick_recv.next().await.expect("expected price tick");

    assert!(matches!(
        received_tick.payload,
        PriceStreamPayload::BitfinexBtcUsdSwapPricePayload(_)
    ));

    Ok(())
}
