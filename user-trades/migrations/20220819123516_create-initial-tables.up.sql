CREATE TYPE user_trade_unit AS ENUM ('sats', 'synth_cents');

CREATE TABLE user_trades (
  idx SERIAL PRIMARY KEY,
  uuid UUID UNIQUE NOT NULL,
  buy_amount NUMERIC NOT NULL,
  buy_unit user_trade_unit NOT NULL,
  sell_amount NUMERIC NOT NULL,
  sell_unit user_trade_unit NOT NULL,
  created_at TIMESTAMP WITH time zone NOT NULL DEFAULT now()
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
  unit user_trade_unit PRIMARY KEY,
  current_balance NUMERIC NOT NULL,
  last_trade_idx INTEGER REFERENCES user_trades (idx),
  updated_at TIMESTAMP WITH time zone NOT NULL DEFAULT now()
);

INSERT INTO user_trade_balances VALUES ('sats', 0, NULL, now());
INSERT INTO user_trade_balances VALUES ('synth_cents', 0, NULL, now());
