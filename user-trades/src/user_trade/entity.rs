use rust_decimal::Decimal;

shared::entity_id! { user_trade_id: UserTradeId }

#[derive(Clone, PartialEq, Eq, Hash, Copy, sqlx::Type)]
#[sqlx(type_name = "user_trade_unit", rename_all = "snake_case")]
pub enum UserTradeUnit {
    Sats,
    SynthCents,
}

pub struct NewUserTrade {
    pub(super) id: UserTradeId,
    pub(super) buy_unit: UserTradeUnit,
    pub(super) buy_amount: Decimal,
    pub(super) sell_unit: UserTradeUnit,
    pub(super) sell_amount: Decimal,
}

impl NewUserTrade {
    pub fn new(
        buy_unit: UserTradeUnit,
        buy_amount: Decimal,
        sell_unit: UserTradeUnit,
        sell_amount: Decimal,
    ) -> Self {
        Self {
            id: UserTradeId::new(),
            buy_unit,
            buy_amount,
            sell_unit,
            sell_amount,
        }
    }
}

pub struct UserTrade {
    pub id: UserTradeId,
    pub idx: i32,
    pub buy_unit: UserTradeUnit,
    pub buy_amount: Decimal,
    pub sell_unit: UserTradeUnit,
    pub sell_amount: Decimal,
}
