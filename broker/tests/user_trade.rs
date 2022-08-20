use broker::user_trade::*;
use rust_decimal_macros::dec;
use sqlx::PgPool;

lazy_static::lazy_static! {
    static ref POOL: PgPool =
    PgPool::connect_lazy("postgres://stablesats:stablesats@127.0.0.1:5432/stablesats-broker").expect("connect to db in user_trade test");
}

#[tokio::test]
async fn create_user_trade() -> anyhow::Result<()> {
    let trades = UserTrades::new(POOL.clone());
    let trade = trades
        .persist_new(NewUserTrade::new(
            UserTradeUnit::Sats,
            dec!(1000),
            UserTradeUnit::SynthCents,
            dec!(10),
        ))
        .await?;
    assert!(trade.idx > 0);
    Ok(())
}
