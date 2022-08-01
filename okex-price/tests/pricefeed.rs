use serde::*;
use okex_price::pricefeed::OkexPriceFeed;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

struct ChannelArgs {
    channel: String,
    instId: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SubscriptionResponse {
    arg: ChannelArgs,
}

#[tokio::test]
async fn test() -> Result<(), anyhow::Error>{
    let wss_url = String::from("wss://ws.okx.com:8443/ws/v5/public");

    let okex_price_feed = OkexPriceFeed::connect(wss_url).await;

    print!("{:?}", okex_price_feed);
    // let sub_response = okex_price_feed::subscribe_to_btcusdswap().await?;

    // let first_resp = sub_response::next().await?;

    // let second_resp = sub_response::next().await?;

    // let ticker_channel = ChannelArgs {
    //     channel: String::from("tickers"),
    //     instId: String::from("BTC-USD-SWAP")
    // };

    // let expected_ticker_response = SubscriptionResponse { arg: ticker_channel};

    // assert_eq!(first_resp, expected_ticker_response);
  
    Ok(())
}
