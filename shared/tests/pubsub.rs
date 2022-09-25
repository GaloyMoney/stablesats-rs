use futures::future;
use futures::stream::{self, StreamExt};
use serde::*;
use stablesats_shared::{payload, pubsub::*};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestMessage {
    test: String,
    #[serde(with = "serialize_as_string")]
    value: u64,
}

payload! { TestMessage, "pubsub.test.message" }

#[tokio::test]
async fn pubsub() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config).await?;
    assert!(subscriber.time_since_last_msg().await.is_none());
    let mut stream = subscriber.subscribe::<TestMessage>().await?;
    let msg = TestMessage {
        test: "test".to_string(),
        value: u64::MAX,
    };
    publisher.publish(msg.clone()).await?;
    let received = stream.next().await;
    assert!(subscriber.time_since_last_msg().await.is_some());
    assert_eq!(msg, received.unwrap().payload);
    Ok(())
}

#[tokio::test]
async fn throttle_price_publishing() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config).await?;
    let mut stream = subscriber.subscribe::<TestMessage>().await?;
    let msg = TestMessage {
        test: "test".to_string(),
        value: u64::MAX,
    };

    publisher.publish_price(msg.clone()).await?;
    // 1. Spawn thread publishing okex messages
    // 2. Sleep for 1 minute
    // 3. subscribe until none
    // 4. count shouls be <= 30

    // let received = stream.next().await;
    // assert_eq!(msg, received.unwrap().payload);
    Ok(())
}
