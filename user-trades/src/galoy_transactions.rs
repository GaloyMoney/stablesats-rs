use sqlx::PgPool;

pub struct GaloyTransactions {
    pool: PgPool,
}

impl GaloyTransactions {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
