#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::broadcast;
use tracing::instrument;

mod balances;
mod constants;
mod error;
mod templates;

pub use constants::*;
pub use error::*;
pub use templates::*;

use sqlx_ledger::{
    account::NewAccount,
    event::{EventSubscriber, EventSubscriberOpts, SqlxLedgerEvent},
    journal::*,
    Currency, DebitOrCredit, SqlxLedger, SqlxLedgerError,
};
pub use sqlx_ledger::{
    event::{SqlxLedgerEvent as LedgerEvent, SqlxLedgerEventData as LedgerEventData},
    TransactionId as LedgerTxId,
};

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
        Self::create_hedging_journal(&inner).await?;

        Self::external_omnibus_account(&inner).await?;
        Self::stablesats_btc_wallet_account(&inner).await?;
        Self::stablesats_omnibus_account(&inner).await?;
        Self::stablesats_liability_account(&inner).await?;
        Self::hedge_position_omnibus_account(&inner).await?;
        Self::okex_position_account(&inner).await?;

        templates::UserBuysUsd::init(&inner).await?;
        templates::UserSellsUsd::init(&inner).await?;
        templates::RevertUserBuysUsd::init(&inner).await?;
        templates::RevertUserSellsUsd::init(&inner).await?;
        templates::IncreaseExchangePosition::init(&inner).await?;
        templates::DecreaseExchangePosition::init(&inner).await?;

        Ok(Self {
            events: inner.events(EventSubscriberOpts::default()).await?,
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

    #[instrument(name = "ledger.increase_exchange_position", skip(self, tx))]
    pub async fn increase_exchange_position(
        &self,
        tx: Transaction<'_, Postgres>,
        params: IncreaseExchangePositionParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(
                tx,
                LedgerTxId::new(),
                INCREASE_EXCHANGE_POSITION_CODE,
                Some(params),
            )
            .await?;
        Ok(())
    }

    #[instrument(name = "ledger.decrease_exchange_position", skip(self, tx))]
    pub async fn decrease_exchange_position(
        &self,
        tx: Transaction<'_, Postgres>,
        params: DecreaseExchangePositionParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(
                tx,
                LedgerTxId::new(),
                DECREASE_EXCHANGE_POSITION_CODE,
                Some(params),
            )
            .await?;
        Ok(())
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

    #[instrument(name = "ledger.revert_user_buys_usd", skip(self, tx))]
    pub async fn revert_user_buys_usd(
        &self,
        tx: Transaction<'_, Postgres>,
        id: LedgerTxId,
        params: RevertUserBuysUsdParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(tx, id, REVERT_USER_BUYS_USD_CODE, Some(params))
            .await?;
        Ok(())
    }

    #[instrument(name = "ledger.revert_user_sells_usd", skip(self, tx))]
    pub async fn revert_user_sells_usd(
        &self,
        tx: Transaction<'_, Postgres>,
        id: LedgerTxId,
        params: RevertUserSellsUsdParams,
    ) -> Result<(), LedgerError> {
        self.inner
            .post_transaction_in_tx(tx, id, REVERT_USER_SELLS_USD_CODE, Some(params))
            .await?;
        Ok(())
    }

    pub async fn usd_liability_balance_events(
        &self,
    ) -> Result<broadcast::Receiver<SqlxLedgerEvent>, LedgerError> {
        Ok(self
            .events
            .account_balance(STABLESATS_JOURNAL_ID.into(), STABLESATS_LIABILITY_ID.into())
            .await?)
    }
    
    //usd_position_balance_events here. 

    #[instrument(name = "ledger.create_stablesats_journal", skip(ledger))]
    async fn create_stablesats_journal(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_journal = NewJournal::builder()
            .id(STABLESATS_JOURNAL_ID)
            .description("Stablesats journal".to_string())
            .name(STABLESATS_JOURNAL_NAME)
            .build()
            .expect("Couldn't build Stablesats journal");
        match ledger.journals().create(new_journal).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.create_hedging_journal", skip(ledger))]
    async fn create_hedging_journal(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_journal = NewJournal::builder()
            .id(HEDGING_JOURNAL_ID)
            .description("Hedging journal".to_string())
            .name(HEDGING_JOURNAL_NAME)
            .build()
            .expect("Couldn't build Hedging journal");
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

    #[instrument(name = "ledger.hedge_position_omnibus_account", skip_all)]
    async fn hedge_position_omnibus_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(HEDGE_POSITION_OMNIBUS_CODE)
            .id(HEDGE_POSITION_OMNIBUS_ID)
            .name(HEDGE_POSITION_OMNIBUS_CODE)
            .normal_balance_type(DebitOrCredit::Credit)
            .description("Omnibus account for all exchange hedging".to_string())
            .build()
            .expect("Couldn't create hedge position omnibus account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(name = "ledger.okex_position_account", skip_all)]
    async fn okex_position_account(ledger: &SqlxLedger) -> Result<(), LedgerError> {
        let new_account = NewAccount::builder()
            .code(OKEX_POSITION_CODE)
            .id(OKEX_POSITION_ID)
            .name(OKEX_POSITION_CODE)
            .normal_balance_type(DebitOrCredit::Debit)
            .description("Account for okex position".to_string())
            .build()
            .expect("Couldn't create okex position account");
        match ledger.accounts().create(new_account).await {
            Ok(_) | Err(SqlxLedgerError::DuplicateKey(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
