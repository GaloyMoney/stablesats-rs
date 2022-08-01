use anyhow::Ok;
use url::{Url};
use tokio_tungstenite::{connect_async, tungstenite::http::{response, Response}, tungstenite::protocol::Message};

#[derive(Debug)]
pub struct OkexPriceFeed {

}


impl OkexPriceFeed {
   pub async fn connect(url: String)-> Result<(), anyhow::Error> {

        let parsed_url = Url::parse(&url).unwrap();
        let (stream, response ) = connect_async(parsed_url).await?;

        let resp= response.body();

        Ok(())
    }
}
