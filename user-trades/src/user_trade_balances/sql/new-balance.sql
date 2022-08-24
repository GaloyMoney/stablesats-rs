 WITH amounts AS (
   SELECT MAX(idx) as latest_idx, buy_unit_id, SUM(buy_amount) AS to_sub, sell_unit_id, SUM(sell_amount) AS to_add 
   FROM user_trades
   WHERE idx > $1
   GROUP BY GROUPING SETS ((buy_unit_id), (sell_unit_id))
 )
SELECT
  unit_id,
  current_balance + COALESCE(sell_amounts.to_add, 0) - COALESCE(buy_amounts.to_sub, 0) AS new_balance,
  MAX(GREATEST(sell_amounts.latest_idx, buy_amounts.latest_idx)) OVER () AS new_latest_idx
FROM user_trade_balances
LEFT JOIN amounts buy_amounts ON unit_id = buy_amounts.buy_unit_id
LEFT JOIN amounts sell_amounts ON unit_id = sell_amounts.sell_unit_id;
