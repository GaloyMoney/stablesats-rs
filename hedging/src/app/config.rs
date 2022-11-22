use serde::{Deserialize, Serialize};
use std::time::Duration;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingAppConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_okex_poll_frequency")]
    pub okex_poll_frequency: Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_liability: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_position: chrono::Duration,

    #[serde(default = "default_contract_size_cents")]
    pub contract_size_cents: Decimal,
    #[serde(default = "default_minimum_liability_threshold_cents")]
    pub minimum_liability_threshold_cents: Decimal,
    #[serde(default = "default_minimum_transfer_amount_cents")]
    pub minimum_transfer_amount_cents: Decimal,

    #[serde(default = "default_minimum_funding_balance_btc")]
    pub minimum_funding_balance_btc: Decimal,

    #[serde(default = "default_low_bound_ratio_shorting")]
    pub low_bound_ratio_shorting: Decimal,
    #[serde(default = "default_low_safebound_ratio_shorting")]
    pub low_safebound_ratio_shorting: Decimal,
    #[serde(default = "default_high_safebound_ratio_shorting")]
    pub high_safebound_ratio_shorting: Decimal,
    #[serde(default = "default_high_bound_ratio_shorting")]
    pub high_bound_ratio_shorting: Decimal,

    #[serde(default = "default_low_bound_ratio_leverage")]
    pub low_bound_ratio_leverage: Decimal,
    #[serde(default = "default_low_safebound_ratio_leverage")]
    pub low_safebound_ratio_leverage: Decimal,
    #[serde(default = "default_high_safebound_ratio_leverage")]
    pub high_safebound_ratio_leverage: Decimal,
    #[serde(default = "default_high_bound_ratio_leverage")]
    pub high_bound_ratio_leverage: Decimal,

    #[serde(default = "default_deposit_lost_timeout_minutes")]
    pub deposit_lost_timeout_minutes: i64,
}

impl Default for HedgingAppConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
            okex_poll_frequency: default_okex_poll_frequency(),
            unhealthy_msg_interval_liability: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_position: default_unhealthy_msg_interval(),

            contract_size_cents: default_contract_size_cents(),
            minimum_liability_threshold_cents: default_minimum_liability_threshold_cents(),
            minimum_transfer_amount_cents: default_minimum_transfer_amount_cents(),

            minimum_funding_balance_btc: default_minimum_funding_balance_btc(),

            low_bound_ratio_shorting: default_low_bound_ratio_shorting(),
            low_safebound_ratio_shorting: default_low_safebound_ratio_shorting(),
            high_safebound_ratio_shorting: default_high_safebound_ratio_shorting(),
            high_bound_ratio_shorting: default_high_bound_ratio_shorting(),

            low_bound_ratio_leverage: default_low_bound_ratio_leverage(),
            low_safebound_ratio_leverage: default_low_safebound_ratio_leverage(),
            high_safebound_ratio_leverage: default_high_safebound_ratio_leverage(),
            high_bound_ratio_leverage: default_high_bound_ratio_leverage(),

            deposit_lost_timeout_minutes: default_deposit_lost_timeout_minutes(),
        }
    }
}

fn bool_true() -> bool {
    true
}

fn default_okex_poll_frequency() -> Duration {
    Duration::from_secs(10)
}

fn default_unhealthy_msg_interval() -> chrono::Duration {
    chrono::Duration::from_std(Duration::from_secs(20))
        .expect("bad default unhealthy_after_msg_delay")
}

fn default_deposit_lost_timeout_minutes() -> i64 {
    60
}

fn default_contract_size_cents() -> Decimal {
    dec!(10000)
}

fn default_minimum_liability_threshold_cents() -> Decimal {
    default_contract_size_cents() / dec!(2)
}

fn default_minimum_transfer_amount_cents() -> Decimal {
    default_contract_size_cents()
}

fn default_minimum_funding_balance_btc() -> Decimal {
    dec!(1)
}

fn default_low_bound_ratio_shorting() -> Decimal {
    dec!(0.95)
}
fn default_low_safebound_ratio_shorting() -> Decimal {
    dec!(0.98)
}
fn default_high_safebound_ratio_shorting() -> Decimal {
    dec!(1.00)
}
fn default_high_bound_ratio_shorting() -> Decimal {
    dec!(1.03)
}
fn default_low_bound_ratio_leverage() -> Decimal {
    dec!(1.20)
}
fn default_low_safebound_ratio_leverage() -> Decimal {
    dec!(1.80)
}
fn default_high_safebound_ratio_leverage() -> Decimal {
    dec!(2.25)
}
fn default_high_bound_ratio_leverage() -> Decimal {
    dec!(3.00)
}
