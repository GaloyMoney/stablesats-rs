mod price_feed;

#[cfg(test)]
mod tests {
    use crate::price_feed::poll_price;

    #[tokio::test]
    async fn test_get_price() {
        poll_price().await;
    }
}
