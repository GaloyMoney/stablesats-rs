INSERT INTO synth_usd_liability (amount, correlation_id) 
  SELECT $1, $2
  WHERE NOT EXISTS (
    SELECT FROM (
      SELECT amount FROM synth_usd_liability ORDER BY idx DESC LIMIT 1
    ) last_row WHERE amount = $1
  )
RETURNING amount;
