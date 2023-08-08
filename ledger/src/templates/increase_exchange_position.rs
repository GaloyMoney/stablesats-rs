use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncreaseExchangePositionMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub exchange_id: String,
    pub instrument_id: String,
}

#[derive(Debug, Clone)]
pub struct IncreaseExchangePositionParams {
    pub usd_cents_amount: Decimal,
    pub exchange_position_id: uuid::Uuid,
    pub meta: IncreaseExchangePositionMeta,
}

impl IncreaseExchangePositionParams {
    pub fn defs() -> Vec<ParamDefinition> {
        vec![
            ParamDefinition::builder()
                .name("usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("exchange_position_id")
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

impl From<IncreaseExchangePositionParams> for TxParams {
    fn from(
        IncreaseExchangePositionParams {
            usd_cents_amount,
            exchange_position_id,
            meta,
        }: IncreaseExchangePositionParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let mut params = Self::default();
        params.insert("usd_amount", usd_cents_amount / CENTS_PER_USD);
        params.insert("exchange_position_id", exchange_position_id);
        params.insert("meta", meta);
        params.insert("effective", effective);
        params
    }
}

pub struct IncreaseExchangePosition {}

impl IncreaseExchangePosition {
    #[instrument(name = "ledger.increase_exchange_position.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{HEDGING_JOURNAL_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Increase exchange position'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            EntryInput::builder()
                .entry_type("'INCREASE_EXCHANGE_POSITION_USD_CR'")
                .currency("'USD'")
                .account_id("params.exchange_position_id")
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build INCREASE_EXCHANGE_POSITION_USD_CR entry"),
            EntryInput::builder()
                .entry_type("'INCREASE_EXCHANGE_POSITION_USD_DR'")
                .currency("'USD'")
                .account_id(format!("uuid('{HEDGE_POSITION_OMNIBUS_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build INCREASE_EXCHANGE_POSITION_USD_DR entry"),
        ];

        let params = IncreaseExchangePositionParams::defs();
        let template = NewTxTemplate::builder()
            .id(INCREASE_EXCHANGE_POSITION_ID)
            .code(INCREASE_EXCHANGE_POSITION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build INCREASE_EXCHANGE_POSITION_CODE");

        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
