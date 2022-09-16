-- Add down migration script here
ALTER TABLE user_trades
 ADD is_latest BOOLEAN UNIQUE;
