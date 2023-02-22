use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError, TransactionId as LedgerTxId};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertUserSellsUsdMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub btc_tx_id: String,
    pub usd_tx_id: String,
}

#[derive(Debug, Clone)]
pub struct RevertUserSellsUsdParams {
    pub satoshi_amount: Decimal,
    pub usd_cents_amount: Decimal,
    pub initial_ledger_tx_id: LedgerTxId,
    pub meta: RevertUserSellsUsdMeta,
}
impl RevertUserSellsUsdParams {
    pub fn defs() -> Vec<ParamDefinition> {
        vec![
            ParamDefinition::builder()
                .name("btc_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("correlation_id")
                .r#type(ParamDataType::UUID)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::JSON)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("effective")
                .r#type(ParamDataType::DATE)
                .build()
                .unwrap(),
        ]
    }
}

impl From<RevertUserSellsUsdParams> for TxParams {
    fn from(
        RevertUserSellsUsdParams {
            satoshi_amount,
            usd_cents_amount,
            initial_ledger_tx_id,
            meta,
        }: RevertUserSellsUsdParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let mut params = Self::default();
        params.insert("btc_amount", satoshi_amount / SATS_PER_BTC);
        params.insert("usd_amount", usd_cents_amount / CENTS_PER_USD);
        params.insert("correlation_id", initial_ledger_tx_id);
        params.insert("meta", meta);
        params.insert("effective", effective);
        params
    }
}
pub struct RevertUserSellsUsd {}

impl RevertUserSellsUsd {
    #[instrument(name = "ledger.user_sells_usd.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{STABLESATS_JOURNAL_ID}')"))
            .effective("params.effective")
            .correlation_id("params.correlation_id")
            .metadata("params.meta")
            .description("'User sells USD'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            EntryInput::builder()
                .entry_type("'REVERT_USER_SELLS_USD_BTC_DR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{STABLESATS_BTC_WALLET_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.btc_amount * -1")
                .build()
                .expect("Couldn't build REVERT_USER_SELLS_USD_BTC_DR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_USER_SELLS_USD_BTC_CR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{EXTERNAL_OMNIBUS_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.btc_amount * -1")
                .build()
                .expect("Couldn't build REVERT_USER_SELLS_USD_BTC_CR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_USER_SELLS_USD_USD_DR'")
                .currency("'USD'")
                .account_id(format!("uuid('{STABLESATS_LIABILITY_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.usd_amount * -1")
                .build()
                .expect("Couldn't build REVERT_USER_SELLS_USD_USD_DR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_USER_SELLS_USD_USD_CR'")
                .currency("'USD'")
                .account_id(format!("uuid('{STABLESATS_OMNIBUS_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.usd_amount * -1")
                .build()
                .expect("Couldn't build REVERT_USER_SELLS_USD_USD_CR entry"),
        ];

        let params = RevertUserSellsUsdParams::defs();
        let template = NewTxTemplate::builder()
            .id(REVERT_USER_SELLS_USD_ID)
            .code(REVERT_USER_SELLS_USD_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build REVERT_USER_SELLS_USD_CODE");
        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
