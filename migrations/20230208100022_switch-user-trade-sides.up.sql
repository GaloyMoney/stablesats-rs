ALTER TABLE user_trades RENAME COLUMN buy_unit TO old_buy_unit;
ALTER TABLE user_trades RENAME COLUMN buy_amount TO old_buy_amount;

ALTER TABLE user_trades RENAME COLUMN sell_unit TO buy_unit;
ALTER TABLE user_trades RENAME COLUMN sell_amount TO buy_amount;

ALTER TABLE user_trades RENAME COLUMN old_buy_unit TO sell_unit;
ALTER TABLE user_trades RENAME COLUMN old_buy_amount TO sell_amount;
