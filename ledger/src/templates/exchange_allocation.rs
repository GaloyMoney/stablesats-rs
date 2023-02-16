use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAllocationMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ExchangeAllocationParams {
    pub okex_allocation_amount: Decimal,
    pub meta: ExchangeAllocationMeta,
}

impl ExchangeAllocationParams {
    pub fn defs() -> Vec<ParamDefinition> {
        vec![
            ParamDefinition::builder()
                .name("okex_allocation_amount")
                .r#type(ParamDataType::DECIMAL)
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

impl From<ExchangeAllocationParams> for TxParams {
    fn from(
        ExchangeAllocationParams {
            okex_allocation_amount,
            meta,
        }: ExchangeAllocationParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let mut params = Self::default();
        params.insert("okex_allocation_amount", okex_allocation_amount);
        params.insert("meta", meta);
        params.insert("effective", effective);
        params
    }
}
pub struct ExchangeAllocation {}

impl ExchangeAllocation {
    #[instrument(name = "ledger.user_buys_usd.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{STABLESATS_JOURNAL_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'User buys USD'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            EntryInput::builder()
                .entry_type("'EXCHANGE_ALLOCATION_LIABILITY_DR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{STABLESATS_LIABILITY_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.okex_allocation_amount")
                .build()
                .expect("Couldn't build EXCHANGE_ALLOCATION_LIABILITY_DR entry"),
            EntryInput::builder()
                .entry_type("'EXCHANGE_ALLOCATION_OKEX_CR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{DERIVATIVE_ALLOCATIONS_OKEX_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.okex_allocation_amount")
                .build()
                .expect("Couldn't build EXCHANGE_ALLOCATION_OKEX_CR entry"),
        ];

        let params = ExchangeAllocationParams::defs();
        let template = NewTxTemplate::builder()
            .id(EXCHANGE_ALLOCATION_ID)
            .code(EXCHANGE_ALLOCATION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build EXCHANGE_ALLOCATION_CODE");
        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
