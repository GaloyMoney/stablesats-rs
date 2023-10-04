CREATE TYPE direction_enum AS ENUM ('BuyCents', 'SellCents');

CREATE TABLE stablesats_quote (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    direction direction_enum NOT NULL,
    immediate_execution BOOLEAN NOT NULL,
    sat_amount BIGINT NOT NULL,
    cent_amount BIGINT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE stablesats_quote_events (
    id UUID REFERENCES stablesats_quote(id) NOT NULL,
    sequence INT NOT NULL,
    event_type VARCHAR NOT NULL,
    event JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(id, sequence)
);
