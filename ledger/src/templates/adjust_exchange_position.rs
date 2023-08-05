use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustExchangePositionMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdjustExchangePositionParams {
    pub usd_cents_amount: Decimal,
    pub exchange_id: String,
    pub instrument_id: String,
    pub meta: AdjustExchangePositionMeta,
}

impl AdjustExchangePositionParams {
    pub fn defs() -> Vec<ParamDefinition> {
        vec![
            ParamDefinition::builder()
                .name("usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("meta")
                .r#type(ParamDataType::JSON)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("exchange_id")
                .r#type(ParamDataType::STRING)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("instrument_id")
                .r#type(ParamDataType::STRING)
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

impl From<AdjustExchangePositionParams> for TxParams {
    fn from(
        AdjustExchangePositionParams {
            usd_cents_amount,
            exchange_id,
            instrument_id,
            meta,
        }: AdjustExchangePositionParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let mut params = Self::default();
        params.insert("usd_amount", usd_cents_amount / CENTS_PER_USD);
        params.insert("exchange_id", exchange_id);
        params.insert("instrument_id", instrument_id);
        params.insert("meta", meta);
        params.insert("effective", effective);
        params
    }
}

pub struct AdjustExchangePosition {}

impl AdjustExchangePosition {
    #[instrument(name = "ledger.adjust_exchange_position.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{HEDGE_POSITION_OMNIBUS_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Adjust exchange position'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            EntryInput::builder()
                .entry_type("'ADJUST_OKEX_POSITION_USD_CR'")
                .currency("'USD'")
                .account_id(format!("uuid('{HEDGE_POSITION_OMNIBUS_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build ADJUST_OKEX_POSITION_USD_CR entry"),
            EntryInput::builder()
                .entry_type("'ADJUST_OKEX_POSITION_USD_DR'")
                .currency("'USD'")
                .account_id(format!("uuid('{OKEX_POSITION_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build ADJUST_OKEX_POSITION_USD_DR entry"),
        ];

        let params = AdjustExchangePositionParams::defs();
        let template = NewTxTemplate::builder()
            .id(ADJUST_EXCHANGE_POSITION_ID)
            .code(ADJUST_EXCHANGE_POSITION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build HEDGE_POSITION_OMNIBUS_CODE");

        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
