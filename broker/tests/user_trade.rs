use broker::user_trade::*;

#[tokio::test]
async fn create_user_trade() -> anyhow::Result<()> {
    let trades = UserTrades::new();
    let id = trades.persist_new(UserTrade::new());
    Ok(())
}
