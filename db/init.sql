-- Schema initialization for telemetry ingestor

CREATE SCHEMA IF NOT EXISTS public;

CREATE TABLE IF NOT EXISTS vessel_register_table (
    vessel_id TEXT PRIMARY KEY,
    name TEXT,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS signal_register_table (
    signal_name TEXT PRIMARY KEY,
    signal_type TEXT CHECK (signal_type IN ('digital', 'analog'))
);

CREATE TABLE IF NOT EXISTS main_raw (
    id BIGSERIAL PRIMARY KEY,
    vessel_id TEXT,
    timestamp_utc TIMESTAMPTZ,
    signal_name TEXT,
    signal_value DOUBLE PRECISION,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS filtered_raw (
    id BIGSERIAL PRIMARY KEY,
    vessel_id TEXT,
    timestamp_utc TIMESTAMPTZ,
    signal_name TEXT,
    signal_value DOUBLE PRECISION,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS server_metrics (
    id BIGSERIAL PRIMARY KEY,
    vessel_id TEXT,
    validation_ms BIGINT,
    ingestion_ms BIGINT,
    total_ms BIGINT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Seed signals Signal_1..Signal_200 (1..50 digital, 51..200 analog)
DO $$
BEGIN
    FOR i IN 1..50 LOOP
        INSERT INTO signal_register_table(signal_name, signal_type)
        VALUES (format('Signal_%s', i), 'digital')
        ON CONFLICT (signal_name) DO NOTHING;
    END LOOP;

    FOR i IN 51..200 LOOP
        INSERT INTO signal_register_table(signal_name, signal_type)
        VALUES (format('Signal_%s', i), 'analog')
        ON CONFLICT (signal_name) DO NOTHING;
    END LOOP;
END$$;

-- Seed a couple of vessels
INSERT INTO vessel_register_table (vessel_id, name, is_active) VALUES
    ('1001', 'Vessel A', TRUE),
    ('1002', 'Vessel B', TRUE)
ON CONFLICT (vessel_id) DO NOTHING;
