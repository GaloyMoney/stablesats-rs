use sqlxmq::NamedJob;

use super::job;
pub struct OkexEngine {
    pool: sqlx::PgPool,
}

impl OkexEngine {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub fn add_context_to_job_registry(&self, runner: &mut sqlxmq::JobRegistry) {
        runner.set_context(self.pool.clone());
    }

    pub fn register_jobs(jobs: &mut Vec<&'static NamedJob>, channels: &mut Vec<&str>) {
        jobs.push(job::adjust_hedge);
        jobs.push(job::poll_okex);
        jobs.push(job::adjust_funding);
        channels.push("hedging.okex");
    }
}
