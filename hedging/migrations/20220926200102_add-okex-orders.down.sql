DROP TABLE okex_orders;

CREATE TABLE exchanges (
  id SERIAL PRIMARY KEY,
  name VARCHAR(20) UNIQUE NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
INSERT INTO exchanges (name) VALUES ('okex');

CREATE TABLE hedging_instruments (
    id SERIAL PRIMARY KEY,
    exchange_id VARCHAR(20) NOT NULL,
    name VARCHAR(20) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
INSERT INTO hedging_instruments (exchange_id, name) SELECT id, 'BTC-USD-SWAP' FROM exchanges WHERE name = 'okex';

CREATE TABLE hedging_adjustments (
  idx SERIAL PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  instrument_id INTEGER NOT NULL REFERENCES hedging_instruments(id),
  exchange_ref VARCHAR(20),
  action VARCHAR(20) NOT NULL,
  size NUMERIC,
  unit VARCHAR(20),
  size_usd_value NUMERIC,
  target_usd_value NUMERIC NOT NULL,
  position_usd_value_before_adjustment NUMERIC NOT NULL,
  position_usd_value_after_adjustment NUMERIC NOT NULL,
  executed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
