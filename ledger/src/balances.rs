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

impl<'a> Balances<'a> {
    pub async fn stablesats_liability(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_JOURNAL_ID, STABLESATS_LIABILITY_ID, self.usd)
            .await
    }

    #[instrument(
        name = "ledger.balances.target_liability_in_cents",
        skip(self),
        fields(liability),
        err
    )]
    pub async fn target_liability_in_cents(&self) -> Result<SyntheticCentLiability, LedgerError> {
        let liability = self.stablesats_liability().await?;
        let res = SyntheticCentLiability::try_from(
            liability.map(|l| l.settled()).unwrap_or(Decimal::ZERO) * CENTS_PER_USD,
        )
        .expect("usd liability has wrong sign");
        tracing::Span::current().record("liability", &tracing::field::display(res));
        Ok(res)
    }

    pub async fn stablesats_btc_wallet(&self) -> Result<Option<AccountBalance>, LedgerError> {
        self.get_ledger_account_balance(STABLESATS_JOURNAL_ID, STABLESATS_BTC_WALLET_ID, self.btc)
            .await
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
