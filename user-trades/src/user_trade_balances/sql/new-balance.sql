 WITH buy_amounts AS (
   SELECT MAX(idx) as max_buy, buy_unit, SUM(buy_amount) AS to_sub FROM user_trades WHERE idx > $1 GROUP BY buy_unit
 ),
 sell_amounts AS (
   SELECT MAX(idx) as max_sell, sell_unit, SUM(sell_amount) AS to_add FROM user_trades WHERE idx > $1 GROUP BY sell_unit
 )
SELECT
  unit AS "unit: _",
  current_balance + COALESCE(to_add, 0) - COALESCE(to_sub, 0) AS new_balance,
  MAX(GREATEST(max_buy, max_sell)) OVER () AS new_latest_idx
FROM user_trade_balances
LEFT JOIN buy_amounts ON unit=buy_unit
LEFT JOIN sell_amounts ON unit=sell_unit;
