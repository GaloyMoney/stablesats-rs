CREATE TYPE UserTradeUnit AS ENUM ('usd_cent', 'satoshi');

ALTER TABLE user_trades ADD COLUMN buy_unit UserTradeUnit NOT NULL DEFAULT 'usd_cent';
ALTER TABLE user_trades ADD COLUMN sell_unit UserTradeUnit NOT NULL DEFAULT 'usd_cent';

UPDATE user_trades SET buy_unit = 'usd_cent', sell_unit = 'satoshi'
WHERE id in (
  SELECT t.id FROM user_trades t JOIN user_trade_units u ON t.buy_unit_id = u.id WHERE u.name = 'synthetic_cent'
);

UPDATE user_trades SET buy_unit = 'satoshi', sell_unit = 'usd_cent'
WHERE id in (
  SELECT t.id FROM user_trades t JOIN user_trade_units u ON t.sell_unit_id = u.id WHERE u.name = 'synthetic_cent'
);

ALTER TABLE user_trades ALTER COLUMN buy_unit DROP DEFAULT;
ALTER TABLE user_trades ALTER COLUMN sell_unit DROP DEFAULT;
ALTER TABLE user_trades DROP COLUMN buy_unit_id;
ALTER TABLE user_trades DROP COLUMN sell_unit_id;
DROP TABLE user_trade_units;
