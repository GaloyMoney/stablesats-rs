shared::entity_id! { user_trade_id: UserTradeId }

pub struct UserTrade {
    pub(super) id: UserTradeId
}

impl UserTrade {
    pub fn new() -> Self {
        Self {
            id: UserTradeId::new()
        }
    }
}
