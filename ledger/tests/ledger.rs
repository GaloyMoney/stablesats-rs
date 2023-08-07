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
async fn adjust_exchange_position() -> anyhow::Result<()> {
    let pool = init_pool().await?;
    let ledger = Ledger::init(&pool).await?;

    ledger
        .balances()
        .exchange_position_account_balance(OKEX_POSITION_ID)
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);

    ledger
        .increase_exchange_position(
            pool.begin().await?,
            IncreaseExchangePositionParams {
                usd_cents_amount: dec!(10000),
                exchange_position_id: OKEX_POSITION_ID,
                meta: IncreaseExchangePositionMeta {
                    timestamp: chrono::Utc::now(),
                    exchange_id: "okex".to_string(),
                    instrument_id: "BTC-USD-SWAP".to_string(),
                },
            },
        )
        .await?;

    let after_increase_exchange_position_account_balance = ledger
        .balances()
        .exchange_position_account_balance(OKEX_POSITION_ID)
        .await?
        .unwrap()
        .settled();

    let after_increase_hedging_omnibus_account_balance = ledger
        .balances()
        .hedge_position_omnibus_account_balance()
        .await?
        .unwrap()
        .settled();

    assert_eq!(
        after_increase_exchange_position_account_balance,
        after_increase_hedging_omnibus_account_balance
    );

    ledger
        .decrease_exchange_position(
            pool.begin().await?,
            DecreaseExchangePositionParams {
                usd_cents_amount: dec!(10000),
                exchange_position_id: OKEX_POSITION_ID,
                meta: DecreaseExchangePositionMeta {
                    timestamp: chrono::Utc::now(),
                    exchange_id: "okex".to_string(),
                    instrument_id: "BTC-USD-SWAP".to_string(),
                },
            },
        )
        .await?;

    let after_decrease_exchange_position_account_balance = ledger
        .balances()
        .exchange_position_account_balance(OKEX_POSITION_ID)
        .await?
        .unwrap()
        .settled();

    let after_decrease_hedging_omnibus_account_balance = ledger
        .balances()
        .hedge_position_omnibus_account_balance()
        .await?
        .unwrap()
        .settled();

    assert_eq!(
        after_decrease_exchange_position_account_balance,
        after_decrease_hedging_omnibus_account_balance
    );
    Ok(())
}
