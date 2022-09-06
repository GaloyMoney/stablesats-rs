CREATE TABLE user_trade_units (
    id SERIAL PRIMARY KEY,
    name VARCHAR(20) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO user_trade_units (name) VALUES ('satoshi'), ('synthetic_cent');

CREATE TABLE user_trades (
  id SERIAL PRIMARY KEY,
  buy_amount NUMERIC NOT NULL,
  buy_unit_id INTEGER NOT NULL REFERENCES user_trade_units(id),
  sell_amount NUMERIC NOT NULL,
  sell_unit_id INTEGER NOT NULL REFERENCES user_trade_units(id),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE FUNCTION notify_user_trades() RETURNS TRIGGER AS $$
BEGIN
  NOTIFY user_trades;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_trades AFTER INSERT ON user_trades
  FOR EACH STATEMENT EXECUTE FUNCTION notify_user_trades();

CREATE TABLE user_trade_balances (
  unit_id INTEGER PRIMARY KEY REFERENCES user_trade_units(id),
  current_balance NUMERIC NOT NULL,
  last_trade_id INTEGER REFERENCES user_trades (id),
  updated_at TIMESTAMP WITH time zone NOT NULL DEFAULT now()
);

INSERT INTO user_trade_balances (unit_id, current_balance, last_trade_id)
  SELECT id, 0, NULL FROM user_trade_units;

CREATE TABLE galoy_transactions (
  id SERIAL PRIMARY KEY,
  tx_cursor VARCHAR(30) UNIQUE NOT NULL,
  btc_tx_id VARCHAR(30) UNIQUE NOT NULL,
  usd_tx_id VARCHAR(30) UNIQUE NOT NULL,
  btc_amount NUMERIC NOT NULL,
  usd_amount NUMERIC NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL
);
