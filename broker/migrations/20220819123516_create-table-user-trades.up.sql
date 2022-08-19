-- Add up migration script here
CREATE TABLE user_trades (
  idx SERIAL PRIMARY KEY,
  uuid UUID NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now()
);
