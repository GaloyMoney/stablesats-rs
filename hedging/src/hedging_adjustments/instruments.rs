use sqlx::PgPool;
use std::collections::HashMap;

use crate::error::HedgingError;

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub enum HedgingInstrument {
    OkexBtcUsdSwap,
}

impl TryFrom<&str> for HedgingInstrument {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "BTC-USD-SWAP" => Ok(Self::OkexBtcUsdSwap),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct HedgingInstruments {
    inner: HashMap<HedgingInstrument, i32>,
}

impl HedgingInstruments {
    pub async fn load(pool: &PgPool) -> Result<Self, HedgingError> {
        let res = sqlx::query!("SELECT id, name FROM hedging_instruments")
            .fetch_all(pool)
            .await?;

        let mut inner = HashMap::new();
        for row in res {
            if let Ok(unit) = HedgingInstrument::try_from(row.name.as_str()) {
                inner.insert(unit, row.id);
            }
        }

        Ok(Self { inner })
    }

    pub fn get_id(&self, unit: HedgingInstrument) -> i32 {
        *self
            .inner
            .get(&unit)
            .expect("HedgingInstruments.get_id - not found")
    }
}
