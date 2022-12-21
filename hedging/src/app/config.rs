use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
    #[serde(default)]
    pub health: HedgingAppHealthConfig,

    #[serde(default)]
    pub hedging: HedgingSectionConfig,

    #[serde(default)]
    pub funding: FundingSectionConfig,
}

impl Default for HedgingAppConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
            okex_poll_frequency: default_okex_poll_frequency(),
            health: HedgingAppHealthConfig::default(),

            hedging: HedgingSectionConfig::default(),
            funding: FundingSectionConfig::default(),
        }
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingAppHealthConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_liability: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_position: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_price: chrono::Duration,
}

impl Default for HedgingAppHealthConfig {
    fn default() -> Self {
        Self {
            unhealthy_msg_interval_liability: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_position: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_price: default_unhealthy_msg_interval(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingSectionConfig {
    #[serde(default = "default_low_bound_ratio_shorting")]
    pub low_bound_ratio_shorting: Decimal,
    #[serde(default = "default_low_safebound_ratio_shorting")]
    pub low_safebound_ratio_shorting: Decimal,
    #[serde(default = "default_high_safebound_ratio_shorting")]
    pub high_safebound_ratio_shorting: Decimal,
    #[serde(default = "default_high_bound_ratio_shorting")]
    pub high_bound_ratio_shorting: Decimal,

    #[serde(default = "default_minimum_liability_threshold_cents")]
    pub minimum_liability_threshold_cents: Decimal,
}
impl Default for HedgingSectionConfig {
    fn default() -> Self {
        Self {
            low_bound_ratio_shorting: default_low_bound_ratio_shorting(),
            low_safebound_ratio_shorting: default_low_safebound_ratio_shorting(),
            high_safebound_ratio_shorting: default_high_safebound_ratio_shorting(),
            high_bound_ratio_shorting: default_high_bound_ratio_shorting(),
            minimum_liability_threshold_cents: default_minimum_liability_threshold_cents(),
        }
    }
}

fn default_minimum_liability_threshold_cents() -> Decimal {
    dec!(5000)
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

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingSectionConfig {
    #[serde(default = "default_minimum_transfer_amount_cents")]
    pub minimum_transfer_amount_cents: Decimal,

    #[serde(default = "default_minimum_funding_balance_btc")]
    pub minimum_funding_balance_btc: Decimal,

    #[serde(default = "default_low_bound_ratio_leverage")]
    pub low_bound_ratio_leverage: Decimal,
    #[serde(default = "default_low_safebound_ratio_leverage")]
    pub low_safebound_ratio_leverage: Decimal,
    #[serde(default = "default_high_safebound_ratio_leverage")]
    pub high_safebound_ratio_leverage: Decimal,
    #[serde(default = "default_high_bound_ratio_leverage")]
    pub high_bound_ratio_leverage: Decimal,
    #[serde(default = "default_high_bound_buffer_percentage")]
    pub high_bound_buffer_percentage: Decimal,

    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_deposit_lost_timeout_seconds")]
    pub deposit_lost_timeout_seconds: chrono::Duration,
}
impl Default for FundingSectionConfig {
    fn default() -> Self {
        Self {
            minimum_transfer_amount_cents: default_minimum_transfer_amount_cents(),

            minimum_funding_balance_btc: default_minimum_funding_balance_btc(),

            low_bound_ratio_leverage: default_low_bound_ratio_leverage(),
            low_safebound_ratio_leverage: default_low_safebound_ratio_leverage(),
            high_safebound_ratio_leverage: default_high_safebound_ratio_leverage(),
            high_bound_ratio_leverage: default_high_bound_ratio_leverage(),
            high_bound_buffer_percentage: default_high_bound_buffer_percentage(),

            deposit_lost_timeout_seconds: default_deposit_lost_timeout_seconds(),
        }
    }
}

fn default_minimum_transfer_amount_cents() -> Decimal {
    dec!(10000)
}

fn default_minimum_funding_balance_btc() -> Decimal {
    dec!(1)
}

fn default_low_bound_ratio_leverage() -> Decimal {
    dec!(2)
}
fn default_low_safebound_ratio_leverage() -> Decimal {
    dec!(3)
}
fn default_high_safebound_ratio_leverage() -> Decimal {
    dec!(3)
}
fn default_high_bound_ratio_leverage() -> Decimal {
    dec!(4)
}
fn default_high_bound_buffer_percentage() -> Decimal {
    dec!(0.9)
}
fn default_deposit_lost_timeout_seconds() -> chrono::Duration {
    chrono::Duration::seconds(3600)
}
