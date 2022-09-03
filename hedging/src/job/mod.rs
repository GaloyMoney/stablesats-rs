mod adjust_hedge;

use sqlx::{Executor, Postgres};
use sqlxmq::{job, CurrentJob, JobRegistry, OwnedHandle};

use okex_client::OkexClient;

use crate::{error::*, synth_usd_liability::*};

pub async fn start_job_runner(
    pool: sqlx::PgPool,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge]);
    registry.set_context(synth_usd_liability);
    registry.set_context(okex);

    Ok(registry.runner(&pool).run().await?)
}

pub async fn spawn_adjust_hedge<'a>(
    tx: impl Executor<'a, Database = Postgres>,
) -> Result<(), HedgingError> {
    println!("SPAWN");
    adjust_hedge
        .builder()
        .set_channel_name("hedging")
        .spawn(tx)
        .await?;
    Ok(())
}

#[job(name = "adjust_hedge", channel_name = "hedging")]
pub async fn adjust_hedge(
    mut current_job: CurrentJob,
    synth_usd_liability: SynthUsdLiability,
    okex: OkexClient,
) -> Result<(), HedgingError> {
    adjust_hedge::execute(synth_usd_liability, okex).await?;
    current_job.complete().await?;
    Ok(())
}
