use futures::stream::StreamExt;
use serde::*;
use shared::{pubsub::*, payload};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TestMessage {
    test: String,
    #[serde(with = "serialize_as_string")]
    value: u64,
}

payload! { TestMessage, "pubsub.test.message" }

#[tokio::test]
async fn test() -> Result<(), anyhow::Error> {
    let publisher = Publisher::new().await?;
    let subscriber = Subscriber::new().await?;
    let mut stream = Box::pin(subscriber.subscribe::<TestMessage>().await?);
    let msg = TestMessage {
        test: "test".to_string(),
        value: u64::MAX,
    };
    publisher.publish(msg.clone()).await?;
    let received = stream.next().await;
    assert_eq!(msg, received.unwrap().unwrap().payload);
    Ok(())
}
