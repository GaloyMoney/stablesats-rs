CREATE TABLE synth_usd_liability (
  idx SERIAL PRIMARY KEY,
  correlation_id UUID UNIQUE NOT NULL,
  amount NUMERIC NOT NULL,
  recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE last_known_hedging_balance (
  idx SERIAL PRIMARY KEY,
  amount NUMERIC NOT NULL,
  recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
