ALTER TABLE okex_orders ADD CONSTRAINT okex_orders_correlation_id_key UNIQUE (correlation_id);
