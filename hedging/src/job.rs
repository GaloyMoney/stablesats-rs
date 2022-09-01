use sqlxmq::{job, CurrentJob, JobRegistry, OwnedHandle};

use std::error::Error;

use crate::error::*;

pub async fn start_job_runner(pool: sqlx::PgPool) -> Result<OwnedHandle, HedgingError> {
    // Construct a job registry from our single job.
    let mut registry = JobRegistry::new(&[adjust_hedge]);
    // Here is where you can configure the registry
    // registry.set_error_handler(...)

    // And add context
    registry.set_context("Hello");

    Ok(registry
        // Create a job runner using the connection pool.
        .runner(&pool)
        // Here is where you can configure the job runner
        // Aim to keep 10-20 jobs running at a time.
        .set_concurrency(10, 20)
        // Start the job runner in the background.
        .run()
        .await?)
}

#[job(channel_name = "adjust_hedge")]
pub async fn adjust_hedge(
    // The first argument should always be the current job.
    mut current_job: CurrentJob,
    // Additional arguments are optional, but can be used to access context
    // provided via [`JobRegistry::set_context`].
    message: &'static str,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    println!("running job");
    return Err(Box::new(HedgingError::Job("hello".to_string())));
    // Decode a JSON payload
    let who: Option<String> = current_job.json()?;

    // Do some work
    println!("{}, {}!", message, who.as_deref().unwrap_or("world"));

    // Mark the job as complete
    current_job.complete().await?;

    Ok(())
}
