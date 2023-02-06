CREATE TABLE user_trade_balances (
  unit_id INTEGER PRIMARY KEY REFERENCES user_trade_units(id),
  current_balance NUMERIC NOT NULL,
  last_trade_id INTEGER REFERENCES user_trades (id),
  updated_at TIMESTAMP WITH time zone NOT NULL DEFAULT now()
);

INSERT INTO user_trade_balances (unit_id, current_balance, last_trade_id)
  SELECT id, 0, NULL FROM user_trade_units;
