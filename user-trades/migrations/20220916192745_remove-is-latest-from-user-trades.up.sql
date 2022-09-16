-- Add up migration script here
ALTER TABLE user_trades
  DROP COLUMN is_latest;
