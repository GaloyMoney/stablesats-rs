use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
}

pub async fn init_pool(config: DbConfig) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(30)
        .connect(&config.pg_con)
        .await?;
    if config.migrate_on_start {
        sqlx::migrate!("../migrations").run(&pool).await?;
    }
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM mq_payloads WHERE id IN (SELECT id FROM mq_msgs WHERE attempts = 0 AND id = ANY($1))",
        &[
        hedging::job::POLL_OKEX_ID,
        user_trades::job::POLL_GALOY_TRANSACTIONS_ID,
        user_trades::job::PUBLISH_LIABILITY_ID,
        ]
    )
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM mq_msgs WHERE id IN (SELECT id FROM mq_msgs WHERE attempts = 0 AND id = ANY($1))",
        &[
        hedging::job::POLL_OKEX_ID,
        user_trades::job::POLL_GALOY_TRANSACTIONS_ID,
        user_trades::job::PUBLISH_LIABILITY_ID,
        ]
    )
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(pool)
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
        }
    }
}

fn bool_true() -> bool {
    true
}

use sqlx::QueryBuilder;
pub async fn migrate_to_unified_db(
    pool: sqlx::PgPool,
    user_trades_con: &str,
    hedging_con: &str,
) -> anyhow::Result<()> {
    let ut_pool = sqlx::PgPool::connect(user_trades_con).await?;
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM user_trade_balances")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM user_trade_units")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let rows = sqlx::query!("SELECT * FROM user_trade_units")
        .fetch_all(&ut_pool)
        .await?;

    let mut bldr = QueryBuilder::new("INSERT INTO user_trade_units (id, name, created_at)");
    bldr.push_values(rows, |mut builder, row| {
        builder.push_bind(row.id);
        builder.push_bind(row.name);
        builder.push_bind(row.created_at);
    });
    let query = bldr.build();
    query.execute(&pool).await?;
    sqlx::query!(
        "SELECT setval('user_trade_units_id_seq', (SELECT MAX(id) FROM user_trade_units))"
    )
    .fetch_all(&pool)
    .await?;

    let mut first_row = String::new();
    loop {
        let rows = sqlx::query!(
            "SELECT * FROM galoy_transactions WHERE id > $1 ORDER BY id LIMIT 1000",
            first_row
        )
        .fetch_all(&ut_pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        bldr = QueryBuilder::new("INSERT INTO galoy_transactions (id, cursor, is_paired, settlement_amount, settlement_currency, settlement_method, cents_per_unit, amount_in_usd_cents, created_at)");
        bldr.push_values(rows, |mut builder, row| {
            builder.push_bind(row.id.clone());
            builder.push_bind(row.cursor);
            builder.push_bind(row.is_paired);
            builder.push_bind(row.settlement_amount);
            builder.push_bind(row.settlement_currency);
            builder.push_bind(row.settlement_method);
            builder.push_bind(row.cents_per_unit);
            builder.push_bind(row.amount_in_usd_cents);
            builder.push_bind(row.created_at);
            first_row = row.id;
        });
        let query = bldr.build();
        query.execute(&pool).await?;
    }
    let mut first_row = 0;
    loop {
        let rows = sqlx::query!(
            "SELECT * FROM user_trades WHERE id >= $1 ORDER BY id LIMIT 1000",
            first_row
        )
        .fetch_all(&ut_pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        bldr = QueryBuilder::new("INSERT INTO user_trades (id, buy_amount, buy_unit_id, sell_amount, sell_unit_id, external_ref, created_at)");
        bldr.push_values(rows, |mut builder, row| {
            builder.push_bind(row.id);
            builder.push_bind(row.buy_amount);
            builder.push_bind(row.buy_unit_id);
            builder.push_bind(row.sell_amount);
            builder.push_bind(row.sell_unit_id);
            builder.push_bind(row.external_ref);
            builder.push_bind(row.created_at);
            first_row = row.id + 1;
        });
        let query = bldr.build();
        query.execute(&pool).await?;
    }
    sqlx::query!("SELECT setval('user_trades_id_seq', (SELECT MAX(id) FROM user_trades))")
        .fetch_all(&pool)
        .await?;

    let rows = sqlx::query!("SELECT * FROM user_trade_balances")
        .fetch_all(&ut_pool)
        .await?;

    let mut bldr = QueryBuilder::new(
        "INSERT INTO user_trade_balances (unit_id, current_balance, last_trade_id, updated_at)",
    );
    bldr.push_values(rows, |mut builder, row| {
        builder.push_bind(row.unit_id);
        builder.push_bind(row.current_balance);
        builder.push_bind(row.last_trade_id);
        builder.push_bind(row.updated_at);
    });
    let query = bldr.build();
    query.execute(&pool).await?;

    let hedging_pool = sqlx::PgPool::connect(hedging_con).await?;

    first_row = 0;
    loop {
        let rows = sqlx::query!(
            "SELECT * FROM synth_usd_liability WHERE idx >= $1 ORDER BY idx LIMIT 1000",
            first_row
        )
        .fetch_all(&hedging_pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        bldr = QueryBuilder::new(
            "INSERT INTO synth_usd_liability (idx, correlation_id, amount, recorded_at)",
        );
        bldr.push_values(rows, |mut builder, row| {
            builder.push_bind(row.idx);
            builder.push_bind(row.correlation_id);
            builder.push_bind(row.amount);
            builder.push_bind(row.recorded_at);
            first_row = row.idx + 1;
        });
        let query = bldr.build();
        query.execute(&pool).await?;
    }
    sqlx::query!(
        "SELECT setval('synth_usd_liability_idx_seq', (SELECT MAX(idx) FROM synth_usd_liability))"
    )
    .fetch_all(&pool)
    .await?;

    let mut first_row = String::new();
    loop {
        let rows = sqlx::query!(
            "SELECT * FROM okex_orders WHERE client_order_id > $1 ORDER BY client_order_id LIMIT 1000",
            first_row
        )
            .fetch_all(&hedging_pool)
            .await?;
        if rows.is_empty() {
            break;
        }
        bldr = QueryBuilder::new("INSERT INTO okex_orders (client_order_id, correlation_id, instrument, action, unit, size, size_usd_value, target_usd_value, position_usd_value_before_order, complete, lost, created_at, order_id, avg_price, fee, state)");
        bldr.push_values(rows, |mut builder, row| {
            builder.push_bind(row.client_order_id.clone());
            builder.push_bind(row.correlation_id);
            builder.push_bind(row.instrument);
            builder.push_bind(row.action);
            builder.push_bind(row.unit);
            builder.push_bind(row.size);
            builder.push_bind(row.size_usd_value);
            builder.push_bind(row.target_usd_value);
            builder.push_bind(row.position_usd_value_before_order);
            builder.push_bind(row.complete);
            builder.push_bind(row.lost);
            builder.push_bind(row.created_at);
            builder.push_bind(row.order_id);
            builder.push_bind(row.avg_price);
            builder.push_bind(row.fee);
            builder.push_bind(row.state);
            first_row = row.client_order_id;
        });
        let query = bldr.build();
        query.execute(&pool).await?;
    }

    first_row = String::new();
    loop {
        let rows = sqlx::query!(
            "SELECT * FROM okex_transfers WHERE client_transfer_id > $1 ORDER BY client_transfer_id LIMIT 1000",
            first_row
        )
            .fetch_all(&hedging_pool)
            .await?;
        if rows.is_empty() {
            break;
        }
        bldr = QueryBuilder::new("INSERT INTO okex_orders (client_transfer_id, correlation_id, action, currency, amount, fee, transfer_from, transfer_to, target_usd_exposure, current_usd_exposure, trading_btc_used_balance, trading_btc_total_balance, current_usd_btc_price, funding_btc_total_balance, lost, transfer_id, state, created_at)");
        bldr.push_values(rows, |mut builder, row| {
            builder.push_bind(row.client_transfer_id.clone());
            builder.push_bind(row.correlation_id);
            builder.push_bind(row.action);
            builder.push_bind(row.currency);
            builder.push_bind(row.amount);
            builder.push_bind(row.fee);
            builder.push_bind(row.transfer_from);
            builder.push_bind(row.transfer_to);
            builder.push_bind(row.target_usd_exposure);
            builder.push_bind(row.current_usd_exposure);
            builder.push_bind(row.trading_btc_used_balance);
            builder.push_bind(row.trading_btc_total_balance);
            builder.push_bind(row.current_usd_btc_price);
            builder.push_bind(row.funding_btc_total_balance);
            builder.push_bind(row.lost);
            builder.push_bind(row.transfer_id);
            builder.push_bind(row.state);
            builder.push_bind(row.created_at);
            first_row = row.client_transfer_id;
        });
        let query = bldr.build();
        query.execute(&pool).await?;
    }
    Ok(())
}
