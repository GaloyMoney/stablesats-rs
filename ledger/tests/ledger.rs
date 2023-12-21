use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serial_test::file_serial;

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

    let initial_okex_balance = ledger
        .balances()
        .okex_position_account_balance()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);

    ledger
        .adjust_okex_position(
            pool.begin().await?,
            dec!(-10000),
            "okex".to_string(),
            "BTC-USD-SWAP".to_string(),
        )
        .await?;
    let balance_after_first_adjustment = ledger
        .balances()
        .okex_position_account_balance()
        .await?
        .unwrap()
        .settled();
    assert_eq!(
        balance_after_first_adjustment - initial_okex_balance,
        dec!(100)
    );
    ledger
        .adjust_okex_position(
            pool.begin().await?,
            dec!(-9000),
            "okex".to_string(),
            "BTC-USD-SWAP".to_string(),
        )
        .await?;
    let balance_after_second_adjustment = ledger
        .balances()
        .okex_position_account_balance()
        .await?
        .unwrap()
        .settled();
    assert_eq!(
        balance_after_second_adjustment - initial_okex_balance,
        dec!(90)
    );
    Ok(())
}

#[tokio::test]
#[file_serial]
async fn buy_and_sell_quotes() -> anyhow::Result<()> {
    let pool = init_pool().await?;

    let ledger = Ledger::init(&pool).await?;

    let before_liability = ledger
        .balances()
        .quotes_usd_liabilities()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);
    let before_btc = ledger
        .balances()
        .quotes_btc_assets()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);

    ledger
        .buy_usd_quote_accepted(
            pool.begin().await?,
            LedgerTxId::new(),
            BuyUsdQuoteAcceptedParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(500),
                meta: BuyUsdQuoteAcceptedMeta {
                    timestamp: chrono::Utc::now(),
                },
            },
        )
        .await?;
    ledger
        .sell_usd_quote_accepted(
            pool.begin().await?,
            LedgerTxId::new(),
            SellUsdQuoteAcceptedParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(500),
                meta: SellUsdQuoteAcceptedMeta {
                    timestamp: chrono::Utc::now(),
                },
            },
        )
        .await?;

    let end_balance = ledger.balances().quotes_usd_liabilities().await?.unwrap();
    let end_btc = ledger.balances().quotes_btc_assets().await?.unwrap();
    assert_eq!(end_balance.settled(), before_liability);
    assert_eq!(end_btc.settled(), before_btc);

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn adjust_exchange_allocation() -> anyhow::Result<()> {
    let pool = init_pool().await?;
    let ledger = Ledger::init(&pool).await?;

    let initial_okex_balance = ledger
        .balances()
        .okex_allocation_account_balance()
        .await?
        .map(|b| b.settled())
        .unwrap_or(Decimal::ZERO);

    ledger
        .adjust_okex_allocation(pool.begin().await?, dec!(-10000), "okex".to_string())
        .await?;
    let balance_after_first_adjustment = ledger
        .balances()
        .okex_allocation_account_balance()
        .await?
        .unwrap()
        .settled();
    assert_eq!(
        balance_after_first_adjustment - initial_okex_balance,
        dec!(100)
    );
    ledger
        .adjust_okex_allocation(pool.begin().await?, dec!(-9000), "okex".to_string())
        .await?;
    let balance_after_second_adjustment = ledger
        .balances()
        .okex_allocation_account_balance()
        .await?
        .unwrap()
        .settled();
    assert_eq!(
        balance_after_second_adjustment - initial_okex_balance,
        dec!(90)
    );
    Ok(())
}
