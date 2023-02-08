use tracing::instrument;

use crate::error::*;

#[instrument(
    name = "hedging.job.hack_user_trades_lag",
    skip_all,
    fields(lag_count),
    err
)]
pub async fn lag_ok(pool: &sqlx::PgPool) -> Result<bool, HedgingError> {
    let span = tracing::Span::current();
    let count = sqlx::query!("SELECT COUNT(*) FROM user_trades WHERE ledger_tx_id IS NULL")
        .fetch_one(pool)
        .await?;
    span.record("lag_count", count.count);
    if count.count.unwrap_or(2) >= 2 {
        return Ok(false);
    }
    Ok(true)
}
