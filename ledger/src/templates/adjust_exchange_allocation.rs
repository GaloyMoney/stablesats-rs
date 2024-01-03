use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustExchangeAllocationMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdjustExchangeAllocationParams {
    pub okex_allocation_adjustment_usd_cents_amount: Decimal,
    pub bitfinex_allocation_adjustment_usd_cents_amount: Decimal,
    pub meta: AdjustExchangeAllocationMeta,
}

impl AdjustExchangeAllocationParams {
    pub fn defs() -> Vec<ParamDefinition> {
        vec![
            ParamDefinition::builder()
                .name("unallocated_adjustment_usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("unallocated_direction")
                .r#type(ParamDataType::STRING)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("okex_adjustment_usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("okex_direction")
                .r#type(ParamDataType::STRING)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("bitfinex_adjustment_usd_amount")
                .r#type(ParamDataType::DECIMAL)
                .build()
                .unwrap(),
            ParamDefinition::builder()
                .name("bitfinex_direction")
                .r#type(ParamDataType::STRING)
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

impl From<AdjustExchangeAllocationParams> for TxParams {
    fn from(
        AdjustExchangeAllocationParams {
            okex_allocation_adjustment_usd_cents_amount: okex_cents,
            bitfinex_allocation_adjustment_usd_cents_amount: bitfinex_cents,
            meta,
        }: AdjustExchangeAllocationParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let unallocated_cents = (okex_cents + bitfinex_cents) * (Decimal::ZERO - Decimal::ONE);

        let mut params = Self::default();
        params.insert(
            "unallocated_adjustment_usd_amount",
            (unallocated_cents / CENTS_PER_USD).abs(),
        );
        params.insert(
            "unallocated_direction",
            if unallocated_cents >= Decimal::ZERO {
                "CREDIT"
            } else {
                "DEBIT"
            },
        );
        params.insert(
            "okex_adjustment_usd_amount",
            (okex_cents / CENTS_PER_USD).abs(),
        );
        params.insert(
            "okex_direction",
            if okex_cents >= Decimal::ZERO {
                "CREDIT"
            } else {
                "DEBIT"
            },
        );
        params.insert(
            "bitfinex_adjustment_usd_amount",
            (bitfinex_cents / CENTS_PER_USD).abs(),
        );
        params.insert(
            "bitfinex_direction",
            if bitfinex_cents >= Decimal::ZERO {
                "CREDIT"
            } else {
                "DEBIT"
            },
        );
        params.insert("meta", meta);
        params.insert("effective", effective);
        params
    }
}

pub struct AdjustExchangeAllocation {}

impl AdjustExchangeAllocation {
    #[instrument(name = "ledger.adjust_exchange_allocation.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{STABLESATS_JOURNAL_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Adjust exchange allocations'")
            .build()
            .expect("Couldn't build TxInput");

        let entries = vec![
            EntryInput::builder()
                .entry_type("'ADJUST_EXCHANGE_ALLOCATION_UNALLOCATED_ADJUSTMENT'")
                .currency("'USD'")
                .account_id(format!("uuid('{STABLESATS_LIABILITY_ID}')"))
                .direction("params.unallocated_direction")
                .layer("SETTLED")
                .units("params.unallocated_adjustment_usd_amount")
                .build()
                .expect("Couldn't build ADJUST_EXCHANGE_ALLOCATION_CODE entry"),
            EntryInput::builder()
                .entry_type("'ADJUST_EXCHANGE_ALLOCATION_OKEX_ADJUSTMENT'")
                .currency("'USD'")
                .account_id(format!("uuid('{OKEX_ALLOCATION_ID}')"))
                .direction("params.okex_direction")
                .layer("SETTLED")
                .units("params.okex_adjustment_usd_amount")
                .build()
                .expect("Couldn't build ADJUST_EXCHANGE_ALLOCATION_OKEX_ADJUSTMENT entry"),
            EntryInput::builder()
                .entry_type("'ADJUST_EXCHANGE_ALLOCATION_BITFINEX_ADJUSTMENT'")
                .currency("'USD'")
                .account_id(format!("uuid('{BITFINEX_ALLOCATION_ID}')"))
                .direction("params.bitfinex_direction")
                .layer("SETTLED")
                .units("params.bitfinex_adjustment_usd_amount")
                .build()
                .expect("Couldn't build ADJUST_EXCHANGE_ALLOCATION_BITFINEX_ADJUSTMENT entry"),
        ];

        let params = AdjustExchangeAllocationParams::defs();
        let template = NewTxTemplate::builder()
            .id(ADJUST_EXCHANGE_ALLOCATION_ID)
            .code(ADJUST_EXCHANGE_ALLOCATION_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build ADJUST_EXCHANGE_ALLOCATION_CODE");

        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
