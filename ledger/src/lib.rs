#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::broadcast;
use tracing::instrument;

mod balances;
mod constants;
mod error;
mod templates;

use constants::*;
pub use error::*;
pub use templates::*;

use sqlx_ledger::{
    account::NewAccount,
    event::{EventSubscriber, SqlxLedgerEvent},
    journal::*,
    Currency, DebitOrCredit, SqlxLedger, SqlxLedgerError,
};
pub use sqlx_ledger::{
    event::{SqlxLedgerEvent as LedgerEvent, SqlxLedgerEventData as LedgerEventData},
    TransactionId as LedgerTxId,
};

const DEFAULT_BUFFER_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct Ledger {
    inner: SqlxLedger,
    events: EventSubscriber,
    usd: Currency,
    btc: Currency,
}

impl Ledger {
    pub async fn init(pool: &PgPool) -> Result<Self, LedgerError> {
        let inner = SqlxLedger::new(pool);

        Self::create_stablesats_journal(&inner).await?;

        Self::external_omnibus_account(&inner).await?;
        Self::stablesats_btc_wallet_account(&inner).await?;
        Self::stablesats_omnibus_account(&inner).await?;
        Self::stablesats_liability_account(&inner).await?;
        Self::derivative_allocation_omnibus_account(&inner).await?;
        Self::derivative_allocation_okex_account(&inner).await?;

        templates::UserBuysUsd::init(&inner).await?;
        templates::UserSellsUsd::init(&inner).await?;
        templates::ExchangeAllocation::init(&inner).await?;

        Ok(Self {
            events: inner.events(DEFAULT_BUFFER_SIZE).await?,
            inner,
            usd: "USD".parse().unwrap(),
            btc: "BTC".parse().unwrap(),
        })
    }

    pub fn balances(&'_ self) -> balances::Balances<'_> {
        balances::Balances {
            inner: &self.inner,
            usd: self.usd,
            btc: self.btc,
        }
    }

    #[instrument(name = "ledger.user_buys_usd", skip(self, tx))]
    pub async fn user_buys_usd(
        &self,
        tx: Transaction<'_, Postgres>,
        id: LedgerTxId,
        params: UserBuysUsdParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(tx, id, USER_BUYS_USD_CODE, Some(params))
            .await?;
        Ok(())
    }

    #[instrument(name = "ledger.user_sells_usd", skip(self, tx))]
    pub async fn user_sells_usd(
        &self,
        tx: Transaction<'_, Postgres>,
        id: LedgerTxId,
        params: UserSellsUsdParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(tx, id, USER_SELLS_USD_CODE, Some(params))
            .await?;
        Ok(())
    }

    #[instrument(name = "ledger.exchange_allocation", skip(self, tx))]
    pub async fn exchange_allocation(
        &self,
        tx: Transaction<'_, Postgres>,
        id: LedgerTxId,
        params: ExchangeAllocationParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(tx, id, EXCHANGE_ALLOCATION_CODE, Some(params))
            .await?;
        Ok(())
    }

    pub async fn usd_liability_balance_events(&self) -> broadcast::Receiver<SqlxLedgerEvent> {
        self.events
            .account_balance(STABLESATS_JOURNAL_ID.into(), STABLESATS_LIABILITY_ID.into())
            .await
    }

    #[instrument(name = "ledger.create_stablesats_journal", skip(ledger))]
    async fn create_stablesats_journal(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_journal = NewJournal::builder()
            .id(STABLESATS_JOURNAL_ID)
            .description("Stablesats journal".to_string())
            .name(STABLESATS_JOURNAL_NAME)
            .build()
            .expect("Couldn't build NewJournal");
        match ledger.journals().create(new_journal).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.external_omnibus_account", skip_all)]
    async fn external_omnibus_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(EXTERNAL_OMNIBUS_CODE)
            .id(EXTERNAL_OMNIBUS_ID)
            .name(EXTERNAL_OMNIBUS_CODE)
            .normal_balance_type(DebitOrCredit::Debit)
            .description("Account for balancing btc coming into wallet".to_string())
            .build()
            .expect("Couldn't create external omnibus account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.stablesats_btc_wallet_account", skip_all)]
    async fn stablesats_btc_wallet_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(STABLESATS_BTC_WALLET)
            .id(STABLESATS_BTC_WALLET_ID)
            .name(STABLESATS_BTC_WALLET)
            .description("Account that records the stablesats btc balance".to_string())
            .build()
            .expect("Couldn't create stablesats btc wallet account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.stablesats_omnibus_account", skip_all)]
    async fn stablesats_omnibus_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(STABLESATS_OMNIBUS)
            .id(STABLESATS_OMNIBUS_ID)
            .name(STABLESATS_OMNIBUS)
            .normal_balance_type(DebitOrCredit::Debit)
            .description("Omnibus account for all stablesats hedging".to_string())
            .build()
            .expect("Couldn't create stablesats omnibus account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.stablesats_omnibus_account", skip_all)]
    async fn stablesats_liability_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(STABLESATS_LIABILITY)
            .id(STABLESATS_LIABILITY_ID)
            .name(STABLESATS_LIABILITY)
            .description("Account for stablesats liability".to_string())
            .build()
            .expect("Couldn't create stablesats liability account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.derivative_allocation_omnibus_account", skip_all)]
    async fn derivative_allocation_omnibus_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(DERIVATIVE_ALLOCATIONS_OMNIBUS)
            .id(DERIVATIVE_ALLOCATIONS_OMNIBUS_ID)
            .name(DERIVATIVE_ALLOCATIONS_OMNIBUS)
            .description("Account for all derivative allocations".to_string())
            .build()
            .expect("Couldn't create stablesats liability account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.derivative_allocation_okex_account", skip_all)]
    async fn derivative_allocation_okex_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(DERIVATIVE_ALLOCATIONS_OKEX)
            .id(DERIVATIVE_ALLOCATIONS_OKEX_ID)
            .name(DERIVATIVE_ALLOCATIONS_OKEX)
            .description("Account for okex derivative allocations".to_string())
            .build()
            .expect("Couldn't create stablesats liability account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
