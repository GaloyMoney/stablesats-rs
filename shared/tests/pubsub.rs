use futures::stream::{self, StreamExt};
use serde::*;
use stablesats_shared::{payload, pubsub::*};
use std::time::{Duration, Instant};
use tokio::time;

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
async fn throttle_publishing() -> anyhow::Result<()> {
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
    let mut price_feed_stream = stream::repeat(msg);

    let now = Instant::now();
    let tp_jh = tokio::spawn(async move {
        while let Some(msg) = price_feed_stream.next().await {
            publisher
                .throttle_publish(msg)
                .await
                .expect("Publisher error");
        }
    });

    let thirty_seconds = Duration::from_secs(30);
    let _ = time::sleep(thirty_seconds).await;
    println!("{:?}", now.elapsed());

    let _ = tp_jh.abort();
    let mut count = 0_u32;
    if now.elapsed() >= thirty_seconds {
        while let Some(_msg) = stream.next().await {
            count = count + 1;
        }
    }

    println!("{}", count);
    assert!(count <= 15);

    Ok(())
}
