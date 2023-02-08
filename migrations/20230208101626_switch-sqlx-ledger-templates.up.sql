UPDATE sqlx_ledger_accounts SET normal_balance_type = 'credit';
UPDATE sqlx_ledger_accounts
  SET normal_balance_type = 'debit' WHERE id = '10000000-0000-0000-0000-000000000001';
  SET normal_balance_type = 'debit' WHERE id = '20000000-0000-0000-0000-000000000001';
