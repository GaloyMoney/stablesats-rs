use rust_decimal::prelude::*;

pub struct Money {
    pub amount: Decimal,
}

impl From<(Decimal, u32)> for Money {
    fn from((mut amount, offset): (Decimal, u32)) -> Self {
        let _ = amount.set_scale(offset);
        Self { amount }
    }
}
