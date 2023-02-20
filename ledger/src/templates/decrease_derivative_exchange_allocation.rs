use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreaseDerivativeExchangeAllocationMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DecreaseDerivativeExchangeAllocationParams {
    pub okex_allocation_amount: Decimal,
    pub meta: DecreaseDerivativeExchangeAllocationMeta,
}

impl DecreaseDerivativeExchangeAllocationParams {
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

impl From<DecreaseDerivativeExchangeAllocationParams> for TxParams {
    fn from(
        DecreaseDerivativeExchangeAllocationParams {
            okex_allocation_amount,
            meta,
        }: DecreaseDerivativeExchangeAllocationParams,
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
pub struct DecreaseDerivativeExchangeAllocation {}

impl DecreaseDerivativeExchangeAllocation {
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
                .entry_type("'EXCHANGE_ALLOCATION_LIABILITY_CR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{STABLESATS_LIABILITY_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.okex_allocation_amount")
                .build()
                .expect("Couldn't build EXCHANGE_ALLOCATION_LIABILITY_DR entry"),
            EntryInput::builder()
                .entry_type("'EXCHANGE_ALLOCATION_OKEX_DR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{DERIVATIVE_ALLOCATIONS_OKEX_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.okex_allocation_amount")
                .build()
                .expect("Couldn't build EXCHANGE_ALLOCATION_OKEX_CR entry"),
        ];

        let params = DecreaseDerivativeExchangeAllocationParams::defs();
        let template = NewTxTemplate::builder()
            .id(DECREASE_DERIVATIVE_EXCHANGE_ALLOCATION_ID)
            .code(DECREASE_DERIVATIVE_EXCHANGE_ALLOCATION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build DECREASE_DERIVATIVE_EXCHANGE_ALLOCATION_CODE");
        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
