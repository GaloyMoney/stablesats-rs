ALTER TABLE galoy_transactions ADD COLUMN unpaired_last_checked_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp();
UPDATE galoy_transactions set unpaired_last_checked_at = created_at;
