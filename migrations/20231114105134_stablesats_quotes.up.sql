CREATE TYPE direction_enum AS ENUM ('BuyCents', 'SellCents');

CREATE TABLE stablesats_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE stablesats_quote_events (
    id UUID REFERENCES stablesats_quotes(id) NOT NULL,
    sequence INT NOT NULL,
    event_type VARCHAR NOT NULL,
    event JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(id, sequence)
);
