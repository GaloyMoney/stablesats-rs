INSERT INTO synth_usd_exposure (amount, correlation_id) 
  SELECT $1, $2
  WHERE NOT EXISTS (
    SELECT FROM (
      SELECT amount FROM synth_usd_exposure ORDER BY idx DESC LIMIT 1
    ) last_row WHERE amount = $1
  )
RETURNING amount;
