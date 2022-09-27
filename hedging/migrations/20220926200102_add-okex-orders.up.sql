-- Add up migration script here

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
