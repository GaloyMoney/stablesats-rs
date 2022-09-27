-- Add up migration script here

CREATE TABLE okex_orders (
  client_order_id VARCHAR(32) PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  instrument_id INTEGER NOT NULL REFERENCES hedging_instruments(id),
  action VARCHAR(20) NOT NULL,
  size NUMERIC NOT NULL,
  unit VARCHAR(20) NOT NULL,
  size_usd_value NUMERIC NOT NULL,
  target_usd_value NUMERIC NOT NULL,
  position_usd_value_before_order NUMERIC NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

  order_id VARCHAR(20),
  avg_price NUMERIC,
  fee NUMERIC,
  state VARCHAR(20)
);
