use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx_ledger::{tx_template::*, SqlxLedger, SqlxLedgerError, TransactionId as LedgerTxId};
use tracing::instrument;

use crate::{constants::*, error::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertSellUsdQuoteAcceptedMeta {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RevertSellUsdQuoteAcceptedParams {
    pub satoshi_amount: Decimal,
    pub usd_cents_amount: Decimal,
    pub correlation_id: LedgerTxId,
    pub meta: RevertSellUsdQuoteAcceptedMeta,
}
impl RevertSellUsdQuoteAcceptedParams {
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

impl From<RevertSellUsdQuoteAcceptedParams> for TxParams {
    fn from(
        RevertSellUsdQuoteAcceptedParams {
            satoshi_amount,
            usd_cents_amount,
            correlation_id,
            meta,
        }: RevertSellUsdQuoteAcceptedParams,
    ) -> Self {
        let effective = meta.timestamp.naive_utc().date();
        let meta = serde_json::to_value(meta).expect("Couldn't serialize meta");
        let mut params = Self::default();
        params.insert("btc_amount", satoshi_amount / SATS_PER_BTC);
        params.insert("usd_amount", usd_cents_amount / CENTS_PER_USD);
        params.insert("meta", meta);
        params.insert("correlation_id", correlation_id);
        params.insert("effective", effective);
        params
    }
}
pub struct RevertSellUsdQuoteAccepted {}

impl RevertSellUsdQuoteAccepted {
    #[instrument(name = "ledger.revert_sell_usd_quote_accepted.init", skip_all)]
    pub async fn init(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let tx_input = TxInput::builder()
            .journal_id(format!("uuid('{STABLESATS_JOURNAL_ID}')"))
            .effective("params.effective")
            .metadata("params.meta")
            .description("'Revert Sell Usd Quote Accepted'")
            .build()
            .expect("Couldn't build TxInput");
        let entries = vec![
            EntryInput::builder()
                .entry_type("'REVERT_SELL_USD_QUOTE_ACCEPTED_BTC_CR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{QUOTES_LIABILITY_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.btc_amount")
                .build()
                .expect("Couldn't build REVERT_SELL_USD_QUOTE_ACCEPTED_BTC_CR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_SELL_USD_QUOTE_ACCEPTED_BTC_DR'")
                .currency("'BTC'")
                .account_id(format!("uuid('{QUOTES_OMNIBUS_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.btc_amount")
                .build()
                .expect("Couldn't build REVERT_SELL_USD_QUOTE_ACCEPTED_BTC_DR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_SELL_USD_QUOTE_ACCEPTED_USD_CR'")
                .currency("'USD'")
                .account_id(format!("uuid('{QUOTES_LIABILITY_ID}')"))
                .direction("CREDIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build REVERT_SELL_USD_QUOTE_ACCEPTED_USD_CR entry"),
            EntryInput::builder()
                .entry_type("'REVERT_SELL_USD_QUOTE_ACCEPTED_USD_DR'")
                .currency("'USD'")
                .account_id(format!("uuid('{QUOTES_OMNIBUS_ID}')"))
                .direction("DEBIT")
                .layer("SETTLED")
                .units("params.usd_amount")
                .build()
                .expect("Couldn't build REVERT_SELL_USD_QUOTE_ACCEPTED_USD_DR entry"),
        ];

        let params = RevertSellUsdQuoteAcceptedParams::defs();
        let template = NewTxTemplate::builder()
            .id(REVERT_SELL_USD_QUOTE_ACCEPTED_ID)
            .code(REVERT_SELL_USD_QUOTE_ACCEPTED_CODE)
            .tx_input(tx_input)
            .entries(entries)
            .params(params)
            .build()
            .expect("Couldn't build REVERT_SELL_USD_QUOTE_ACCEPTED_CODE");
        match ledger.tx_templates().create(template).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
