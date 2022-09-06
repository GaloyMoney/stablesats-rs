use sqlx::PgPool;

#[derive(Clone)]
pub struct GaloyTransactions {
    pool: PgPool,
}

impl GaloyTransactions {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
