use sqlxmq::{job, CurrentJob, JobRegistry, OwnedHandle};

use crate::{error::*, synth_usd_exposure::*};

pub async fn start_job_runner(pool: sqlx::PgPool, synth_usd_exposure: SynthUsdExposure) -> Result<OwnedHandle, HedgingError> {
    let mut registry = JobRegistry::new(&[adjust_hedge]);
    registry.set_context(synth_usd_exposure);

    Ok(registry.runner(&pool).run().await?)
}

pub async fn spawn_adjust_hedge(pool: &sqlx::PgPool) -> Result<(), HedgingError> {
    adjust_hedge
        .builder()
        .set_channel_name("hedging")
        .spawn(pool)
        .await?;
    Ok(())
}

#[job(name = "adjust_hedge", channel_name = "hedging")]
pub async fn adjust_hedge(mut current_job: CurrentJob, synth_usd_exposure: SynthUsdExposure) -> Result<(), HedgingError> {
    let latest_exposure = synth_usd_exposure.get_latest_exposure().await?;
    // use OKEX client here
    // load last known exposure
    // if needed {
      // execute hedge adjustment
      // => if fail then fail job
    // }

    current_job.complete().await?;

    Ok(())
}
