use futures::stream::StreamExt;
use serde::*;
use shared::{payload, pubsub::*};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestMessage {
    test: String,
    #[serde(with = "serialize_as_string")]
    value: u64,
}

payload! { TestMessage, "pubsub.test.message" }

#[tokio::test]
async fn pubsub_test() -> Result<(), anyhow::Error> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config).await?;
    let mut stream = subscriber.subscribe::<TestMessage>().await?;
    let msg = TestMessage {
        test: "test".to_string(),
        value: u64::MAX,
    };
    publisher.publish(msg.clone()).await?;
    let received = stream.next().await;
    assert_eq!(msg, received.unwrap().payload);
    Ok(())
}
