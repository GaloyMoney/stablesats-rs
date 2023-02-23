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

pub struct ExchangeBalances {
    pub okex: Option<AccountBalance>,
}

impl<'a> Balances<'a> {
    #[instrument(name = "ledger.balances.stablesats_liability", skip(self))]
    pub async fn stablesats_liability(&self) -> Result<Decimal, LedgerError> {
        Ok(self
            .get_ledger_account_balance(STABLESATS_LIABILITY_ID, self.usd)
            .await?
            .map(|b| b.settled())
            .unwrap_or(Decimal::ZERO))
    }

    #[instrument(
        name = "ledger.balances.target_liability_in_cents",
        skip(self),
        fields(liability),
        err
    )]
    pub async fn okex_target_liability_in_cents(
        &self,
    ) -> Result<SyntheticCentLiability, LedgerError> {
        let liability = self.exchange_allocations().await?.okex;
        let res = SyntheticCentLiability::try_from(
            liability.map(|l| l.settled()).unwrap_or(Decimal::ZERO) * CENTS_PER_USD,
        )
        .expect("usd liability has wrong sign");
        tracing::Span::current().record("liability", &tracing::field::display(res));
        Ok(res)
    }

    pub async fn stablesats_btc_wallet(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_BTC_WALLET_ID, self.btc)
            .await
    }

    #[instrument(name = "ledger.balances.get_ledger_account_balance", skip(self))]
    pub async fn get_ledger_account_balance(
        &self,
        account_id: impl Into<LedgerAccountId> + std::fmt::Debug,
        currency: Currency,
    ) -> Result<Option<AccountBalance>, LedgerError> {
        Ok(self
            .inner
            .balances()
            .find(STABLESATS_JOURNAL_ID.into(), account_id.into(), currency)
            .await?)
    }

    #[instrument(name = "ledger.get_ledger_account_balance", skip(self))]
    pub async fn exchange_allocations(&self) -> Result<ExchangeBalances, LedgerError> {
        Ok(ExchangeBalances {
            okex: self
                .get_ledger_account_balance(DERIVATIVE_ALLOCATIONS_OKEX_ID, self.usd)
                .await?,
        })
    }

    #[instrument(name = "ledger.balances.currently_allocated_in_all_exchanges", skip(self))]
    pub async fn currently_allocated_in_all_exchanges(&self) -> Result<Decimal, LedgerError> {
        let exchange_allocations = self.exchange_allocations().await?;
        let okex_liability = exchange_allocations
            .okex
            .map(|l| l.settled())
            .unwrap_or(Decimal::ZERO);
        Ok(okex_liability)
    }
}
