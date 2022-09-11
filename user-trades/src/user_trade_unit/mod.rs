use sqlx::PgPool;
use std::collections::HashMap;

use crate::error::UserTradesError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum UserTradeUnit {
    Satoshi,
    SynthCent,
}

impl TryFrom<&str> for UserTradeUnit {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "satoshi" => Ok(UserTradeUnit::Satoshi),
            "synthetic_cent" => Ok(UserTradeUnit::SynthCent),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct UserTradeUnits {
    inner: HashMap<UserTradeUnit, i32>,
}

impl UserTradeUnits {
    pub async fn load(pool: &PgPool) -> Result<Self, UserTradesError> {
        let res = sqlx::query!("SELECT id, name FROM user_trade_units")
            .fetch_all(pool)
            .await?;

        let mut inner = HashMap::new();
        for row in res {
            if let Ok(unit) = UserTradeUnit::try_from(row.name.as_str()) {
                inner.insert(unit, row.id);
            }
        }

        Ok(Self { inner })
    }

    pub fn get_id(&self, unit: UserTradeUnit) -> i32 {
        *self
            .inner
            .get(&unit)
            .expect("UserTradeUnit.get_id - not found")
    }

    pub fn from_id(&self, id: i32) -> UserTradeUnit {
        self.inner
            .iter()
            .find_map(|(k, v)| if *v == id { Some(*k) } else { None })
            .expect("UserTradeUnit.from_id -  not found")
    }
}
