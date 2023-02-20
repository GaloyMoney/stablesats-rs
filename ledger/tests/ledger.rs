use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use stablesats_ledger::*;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5432/pg",);
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

#[tokio::test]
async fn user_buys_and_sells_usd() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let ledger = Ledger::init(&pool).await?;

    let before_liability = ledger
        .balances()
        .stablesats_liability()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);
    let before_btc = ledger
        .balances()
        .stablesats_btc_wallet()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);
    ledger
        .user_buys_usd(
            pool.begin().await?,
            LedgerTxId::new(),
            UserBuysUsdParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(500),
                meta: UserBuysUsdMeta {
                    timestamp: chrono::Utc::now(),
                    btc_tx_id: "btc_tx_id".to_string(),
                    usd_tx_id: "usd_tx_id".to_string(),
                },
            },
        )
        .await?;

    let after_liability = ledger
        .balances()
        .stablesats_liability()
        .await?
        .unwrap()
        .settled();
    let after_btc = ledger
        .balances()
        .stablesats_btc_wallet()
        .await?
        .unwrap()
        .settled();
    assert_eq!(after_liability - before_liability, dec!(5));
    assert_eq!(after_btc - before_btc, dec!(0.01));

    ledger
        .user_sells_usd(
            pool.begin().await?,
            LedgerTxId::new(),
            UserSellsUsdParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(500),
                meta: UserSellsUsdMeta {
                    timestamp: chrono::Utc::now(),
                    btc_tx_id: "btc_tx_id".to_string(),
                    usd_tx_id: "usd_tx_id".to_string(),
                },
            },
        )
        .await?;
    let end_balance = ledger.balances().stablesats_liability().await?.unwrap();
    let end_btc = ledger.balances().stablesats_btc_wallet().await?.unwrap();
    assert_eq!(end_balance.settled(), before_liability);
    assert_eq!(end_btc.settled(), before_btc);

    Ok(())
}

#[tokio::test]
async fn exchange_allocation() -> anyhow::Result<()> {
    let pool = init_pool().await?;
    let ledger = Ledger::init(&pool).await?;

    let okex_allocation_amount = dec!(200);

    ledger
        .increase_derivatives_exchange_allocation(
            pool.begin().await?,
            LedgerTxId::new(),
            IncreaseDerivativeExchangeAllocationParams {
                okex_allocation_amount,
                meta: IncreaseDerivativeExchangeAllocationMeta {
                    timestamp: chrono::Utc::now(),
                },
            },
        )
        .await?;

    ledger
        .decrease_derivatives_exchange_allocation(
            pool.begin().await?,
            LedgerTxId::new(),
            DecreaseDerivativeExchangeAllocationParams {
                okex_allocation_amount,
                meta: DecreaseDerivativeExchangeAllocationMeta {
                    timestamp: chrono::Utc::now(),
                },
            },
        )
        .await?;

    Ok(())
}
