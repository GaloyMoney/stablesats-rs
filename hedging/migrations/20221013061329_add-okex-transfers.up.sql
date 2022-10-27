-- Add up migration script here
CREATE TABLE okex_transfers (
  
  client_transfer_id VARCHAR(32) PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  
  action VARCHAR(20) NOT NULL,
  transfer_type VARCHAR(20) NOT NULL CHECK (transfer_type in ('internal', 'external')),

  currency VARCHAR(20) NOT NULL,
  amount NUMERIC NOT NULL,
  fee NUMERIC NOT NULL,

  transfer_from VARCHAR(128) NULL,
  transfer_to VARCHAR(128) NULL,

  target_usd_exposure NUMERIC NOT NULL,
  current_usd_exposure NUMERIC NOT NULL,
  trading_btc_used_balance NUMERIC NOT NULL,
  trading_btc_total_balance NUMERIC NOT NULL,
  current_usd_btc_price NUMERIC NOT NULL,
  funding_btc_total_balance NUMERIC NOT NULL,

  complete BOOLEAN NOT NULL DEFAULT FALSE,
  lost BOOLEAN NOT NULL DEFAULT FALSE,

  transfer_id VARCHAR(20),
  state VARCHAR(20) NOT NULL CHECK (state in ('success', 'pending, failed')),

  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
