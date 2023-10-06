CREATE TYPE direction_enum AS ENUM ('BuyCents', 'SellCents');

CREATE TABLE stablesats_quote (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid()
);

CREATE TABLE stablesats_quote_events (
    id UUID REFERENCES stablesats_quote(id) NOT NULL,
    sequence INT NOT NULL,
    event_type VARCHAR NOT NULL,
    event JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(id, sequence)
);
