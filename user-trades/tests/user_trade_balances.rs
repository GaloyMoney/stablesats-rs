use chrono::Utc;
use rust_decimal_macros::dec;

use ::user_trades::{user_trade_balances::*, user_trade_unit::*, user_trades::*};

#[tokio::test]
async fn user_trade_balances() -> anyhow::Result<()> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://stablesats:stablesats@{pg_host}:5432/stablesats");
    let pool = sqlx::PgPool::connect(&pg_con).await?;

    let units = UserTradeUnits::load(&pool).await?;
    let balances = UserTradeBalances::new(pool.clone(), units.clone()).await?;
    let original_balances = balances.fetch_all().await?;

    let trades = UserTrades::new(pool.clone(), units);

    let sat_amount = dec!(1000);
    let cent_amount = dec!(10);
    let external_ref = ExternalRef {
        timestamp: Utc::now(),
        btc_tx_id: "btc_tx_id".to_string(),
        usd_tx_id: "usd_tx_id".to_string(),
    };
    let mut tx = pool.begin().await?;
    trades
        .persist_all(
            &mut tx,
            vec![NewUserTrade {
                buy_unit: UserTradeUnit::SynthCent,
                buy_amount: cent_amount,
                sell_unit: UserTradeUnit::Satoshi,
                sell_amount: sat_amount,
                external_ref: external_ref.clone(),
            }],
        )
        .await?;
    tx.commit().await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let new_balances = balances.fetch_all().await?;
    let old_sat_summary = original_balances
        .get(&UserTradeUnit::Satoshi)
        .expect("old sat summary");
    let new_sat_summary = new_balances
        .get(&UserTradeUnit::Satoshi)
        .expect("new sats balance");

    assert_eq!(
        old_sat_summary.current_balance + sat_amount,
        new_sat_summary.current_balance
    );

    let old_cent_summary = original_balances
        .get(&UserTradeUnit::SynthCent)
        .expect("old cent summary");
    let new_cent_summary = new_balances
        .get(&UserTradeUnit::SynthCent)
        .expect("new cents balance");

    assert_eq!(
        old_cent_summary.current_balance - cent_amount,
        new_cent_summary.current_balance
    );

    Ok(())
}
