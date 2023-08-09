use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreaseExchangePositionMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub exchange_id: String,
    pub instrument_id: String,
}

#[derive(Debug, Clone)]
pub struct DecreaseExchangePositionParams {
    pub usd_cents_amount: Decimal,
    pub exchange_position_id: uuid::Uuid,
    pub meta: DecreaseExchangePositionMeta,
}

impl DecreaseExchangePositionParams {
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

impl From<DecreaseExchangePositionParams> for TxParams {
    fn from(
        DecreaseExchangePositionParams {
            usd_cents_amount,
            exchange_position_id,
            meta,
        }: DecreaseExchangePositionParams,
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

pub struct DecreaseExchangePosition {}

impl DecreaseExchangePosition {
    #[instrument(name = "ledger.decrease_exchange_position.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{EXCHANGE_POSITION_JOURNAL_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Decrease exchange position'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            EntryInput::builder()
                .entry_type("'DECREASE_EXCHANGE_POSITION_USD_DR'")
                .currency("'USD'")
                .account_id("params.exchange_position_id")
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build DECREASE_EXCHANGE_POSITION_USD_DR entry"),
            EntryInput::builder()
                .entry_type("'DECREASE_EXCHANGE_POSITION_USD_CR'")
                .currency("'USD'")
                .account_id(format!("uuid('{EXCHANGE_POSITION_OMNIBUS_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build DECREASE_EXCHANGE_POSITION_USD_CR entry"),
        ];

        let params = DecreaseExchangePositionParams::defs();
        let template = NewTxTemplate::builder()
            .id(DECREASE_EXCHANGE_POSITION_ID)
            .code(DECREASE_EXCHANGE_POSITION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build DECREASE_EXCHANGE_POSITION_CODE");

        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
