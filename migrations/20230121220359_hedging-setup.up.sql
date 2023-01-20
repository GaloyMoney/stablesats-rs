CREATE TABLE synth_usd_liability (
  idx SERIAL PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  amount NUMERIC NOT NULL,
  recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE okex_orders (
  client_order_id VARCHAR(32) PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  instrument VARCHAR(32) NOT NULL,
  action VARCHAR(20) NOT NULL,
  unit VARCHAR(20) NOT NULL,
  size NUMERIC,
  size_usd_value NUMERIC,
  target_usd_value NUMERIC NOT NULL,
  position_usd_value_before_order NUMERIC NOT NULL,
  complete BOOLEAN NOT NULL DEFAULT FALSE,
  lost BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

  order_id VARCHAR(20),
  avg_price NUMERIC,
  fee NUMERIC,
  state VARCHAR(20)
);

CREATE TABLE okex_transfers (
  
  client_transfer_id VARCHAR(32) PRIMARY KEY,
  correlation_id UUID NOT NULL,
  
  action VARCHAR(32) NOT NULL CHECK (action in ('transfer-trading-to-funding', 'transfer-funding-to-trading', 'deposit', 'withdraw')),

  currency VARCHAR(16) NOT NULL,
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

  lost BOOLEAN NOT NULL DEFAULT FALSE,

  transfer_id VARCHAR(64),
  state VARCHAR(20) NOT NULL CHECK (state in ('success', 'pending', 'failed', 'deleted')),

  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
