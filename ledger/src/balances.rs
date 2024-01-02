use rust_decimal::Decimal;
use sqlx_ledger::{balance::AccountBalance, AccountId as LedgerAccountId, Currency, SqlxLedger};
use tracing::instrument;

use crate::{constants::*, LedgerError};
use shared::payload::SyntheticCentLiability;

pub struct Balances<'a> {
    pub(super) inner: &'a SqlxLedger,
    pub(super) usd: Currency,
    pub(super) btc: Currency,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LiabilityAllocations {
    pub unallocated_usd: Decimal,
    pub okex_allocation: SyntheticCentLiability,
    pub bitfinex_allocation: SyntheticCentLiability,
    pub total_liability: SyntheticCentLiability,
}

impl<'a> Balances<'a> {
    #[instrument(
        name = "ledger.balances.usd_liability_balances",
        skip(self),
        fields(unallocated, okex, bitfinex, omnibus),
        err,
        ret
    )]
    pub async fn usd_liability_balances(&self) -> Result<LiabilityAllocations, LedgerError> {
        let unallocated_id = sqlx_ledger::AccountId::from(STABLESATS_LIABILITY_ID);
        let okex_id = sqlx_ledger::AccountId::from(OKEX_ALLOCATION_ID);
        let bitfinex_id = sqlx_ledger::AccountId::from(BITFINEX_ALLOCATION_ID);
        let omnibus = sqlx_ledger::AccountId::from(STABLESATS_OMNIBUS_ID);

        let mut balances = self
            .inner
            .balances()
            .find_all(
                STABLESATS_JOURNAL_ID.into(),
                [unallocated_id, okex_id, bitfinex_id, omnibus],
            )
            .await?;
        let ret = LiabilityAllocations {
            unallocated_usd: balances
                .remove(&unallocated_id)
                .and_then(|mut b| b.remove(&self.usd))
                .map(|b| b.settled())
                .unwrap_or(Decimal::ZERO),
            okex_allocation: SyntheticCentLiability::try_from(
                balances
                    .remove(&okex_id)
                    .and_then(|mut b| b.remove(&self.usd))
                    .map(|b| b.settled())
                    .unwrap_or(Decimal::ZERO)
                    * CENTS_PER_USD,
            )
            .expect("usd liability has wrong sign"),
            bitfinex_allocation: SyntheticCentLiability::try_from(
                balances
                    .remove(&bitfinex_id)
                    .and_then(|mut b| b.remove(&self.usd))
                    .map(|b| b.settled())
                    .unwrap_or(Decimal::ZERO)
                    * CENTS_PER_USD,
            )
            .expect("usd liability has wrong sign"),
            total_liability: SyntheticCentLiability::try_from(
                balances
                    .remove(&omnibus)
                    .and_then(|mut b| b.remove(&self.usd))
                    .map(|b| b.settled())
                    .unwrap_or(Decimal::ZERO)
                    * CENTS_PER_USD,
            )
            .expect("usd liability has wrong sign"),
        };
        tracing::Span::current().record(
            "unallocated_usd",
            &tracing::field::display(ret.unallocated_usd),
        );
        tracing::Span::current().record("okex", &tracing::field::display(ret.okex_allocation));
        tracing::Span::current().record(
            "bitfinex",
            &tracing::field::display(ret.bitfinex_allocation),
        );
        tracing::Span::current().record("omnibus", &tracing::field::display(ret.total_liability));
        Ok(ret)
    }

    pub async fn quotes_usd_liabilities(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_JOURNAL_ID, QUOTES_LIABILITIES_ID, self.usd)
            .await
    }

    pub async fn quotes_btc_assets(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_JOURNAL_ID, QUOTES_ASSETS_ID, self.btc)
            .await
    }

    pub async fn stablesats_btc_assets(&self) -> Result<Decimal, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_JOURNAL_ID, STABLESATS_BTC_WALLET_ID, self.btc)
            .await
            .map(|b| b.map(|b| b.settled()).unwrap_or(Decimal::ZERO))
    }

    async fn exchange_position_account_balance(
        &self,
        exchange_position_id: impl Into<LedgerAccountId> + std::fmt::Debug,
    ) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(
            EXCHANGE_POSITION_JOURNAL_ID,
            exchange_position_id,
            self.usd,
        )
        .await
    }

    pub async fn okex_position_account_balance(
        &self,
    ) -> Result<Option<AccountBalance>, LedgerError> {
        self.exchange_position_account_balance(OKEX_POSITION_ID)
            .await
    }

    #[instrument(name = "ledger.get_ledger_account_balance", skip(self))]
    pub async fn get_ledger_account_balance(
        &self,
        journal_id: impl Into<sqlx_ledger::JournalId> + std::fmt::Debug,
        account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
        currency: Currency,
    ) -> Result<Option<AccountBalance>, LedgerError> {
        Ok(self
            .inner
            .balances()
            .find(journal_id.into(), account_id.into(), currency)
            .await?)
    }
}
