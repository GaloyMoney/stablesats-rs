DELETE FROM sqlx_ledger_accounts;
DELETE FROM sqlx_ledger_transactions;
DELETE FROM sqlx_ledger_entries;
DELETE FROM sqlx_ledger_events;
DELETE FROM sqlx_ledger_balances;
DELETE FROM sqlx_ledger_current_balances;
DELETE FROM sqlx_ledger_tx_templates;

UPDATE user_trades SET ledger_tx_id = NULL;
