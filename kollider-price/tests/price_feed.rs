use kollider_price::config::KolliderPriceFeedConfig;
use url::Url;

use shared::{payload::*, pubsub::*};

#[tokio::test]
#[ignore = "currently kolider testnet is flaky"]
async fn publishes_to_price_stream() -> anyhow::Result<()> {
    let (tick_send, mut tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());

    let _ = tokio::spawn(async move {
        let config = KolliderPriceFeedConfig {
            url: Url::parse("wss://testnet.kollider.xyz/v1/ws/").unwrap(),
        };
        let _ = kollider_price::run(config, tick_send).await;
    });

    let received_tick = tick_recv.next().await.expect("expected price tick");

    assert!(matches!(
        received_tick.payload,
        PriceStreamPayload::KolliderBtcUsdSwapPricePayload(_)
    ));

    Ok(())
}
