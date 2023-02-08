#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "UserTradeUnit", rename_all = "snake_case")]
pub enum UserTradeUnit {
    UsdCent,
    Satoshi,
}
