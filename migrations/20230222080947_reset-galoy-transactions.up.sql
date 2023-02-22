DROP TABLE galoy_transactions;

CREATE TABLE galoy_transactions (
  id VARCHAR(60) UNIQUE PRIMARY KEY,
  cursor VARCHAR(60) NOT NULL,
  is_paired BOOLEAN,
  settlement_amount NUMERIC NOT NULL,
  settlement_currency VARCHAR(10) NOT NULL,
  settlement_method VARCHAR(60) NOT NULL,
  direction VARCHAR NOT NULL,
  memo VARCHAR,
  cents_per_unit NUMERIC NOT NULL,
  amount_in_usd_cents NUMERIC NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

ALTER TABLE user_trades ADD COLUMN correction_ledger_tx_id UUID DEFAULT NULL;
