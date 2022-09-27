use futures::future::abortable;
use futures::stream::{self, StreamExt};
use serde::*;
use stablesats_shared::{payload, pubsub::*};
use std::time::{Duration, Instant};
use tokio::{task::spawn, time};

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
#[ignore = "long-running: subscriber stream never yields `None`"]
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
    let price_feed_stream = stream::iter(vec![msg; 20]);

    let now = Instant::now();
    let (task, handle) = abortable(async move {
        while let Some(msg) = price_feed_stream.clone().next().await {
            let _ = publisher.throttle_publish(msg).await;
        }
    });
    spawn(task);

    let thirty_seconds = Duration::from_secs(30);
    let _ = time::sleep(thirty_seconds).await;
    println!("{:?}", now.elapsed());

    let _ = handle.abort();
    let mut count = 0_u32;
    if now.elapsed() >= thirty_seconds {
        while let Some(msg) = stream.next().await {
            println!("{:?}", msg);
            count = count + 1;
        }
    }

    println!("{}", count);
    assert!(count == 12);

    Ok(())
}
