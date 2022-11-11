#![allow(clippy::or_fun_call)]

use futures::{channel::mpsc, stream::StreamExt, SinkExt};
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
        rate_limit_interval: Duration::from_millis(100),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config).await?;
    let stream = subscriber.subscribe::<TestMessage>().await?;
    let msg = TestMessage {
        test: "test".to_string(),
        value: u64::MAX,
    };

    let (mut snd, recv) = mpsc::channel(10000);
    tokio::spawn(async move {
        loop {
            let _ = snd
                .send(publisher.throttle_publish(msg.clone()).await.unwrap())
                .await;
            time::sleep(Duration::from_millis(10)).await;
        }
    });

    let now = Instant::now();
    let msgs: Vec<_> = stream.take(12).collect().await;
    assert!(now.elapsed() >= Duration::from_secs(1));
    assert_eq!(msgs.len(), 12);
    let n_rejected = recv
        .take(100)
        .fold(0, |acc, sent| async move {
            if !sent {
                acc + 1
            } else {
                acc
            }
        })
        .await;
    assert!(n_rejected > 50);
    Ok(())
}
