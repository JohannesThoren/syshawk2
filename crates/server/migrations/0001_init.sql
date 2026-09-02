-- Requires the TimescaleDB extension to be available on the target Postgres instance.
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS probes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    token_hash  TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen   TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS snapshots (
    time              TIMESTAMPTZ NOT NULL,
    probe_id          UUID NOT NULL REFERENCES probes(id) ON DELETE CASCADE,
    hostname          TEXT NOT NULL,
    uptime_secs       BIGINT NOT NULL,
    cpu_global_pct    REAL NOT NULL,
    cpu_per_core_pct  JSONB NOT NULL,
    cpu_core_count    INT NOT NULL,
    cpu_brand         TEXT NOT NULL,
    mem_total_bytes   BIGINT NOT NULL,
    mem_used_bytes    BIGINT NOT NULL,
    swap_total_bytes  BIGINT NOT NULL,
    swap_used_bytes   BIGINT NOT NULL,
    disks             JSONB NOT NULL,
    network           JSONB NOT NULL,
    processes         JSONB NOT NULL,
    load1             DOUBLE PRECISION NOT NULL,
    load5             DOUBLE PRECISION NOT NULL,
    load15            DOUBLE PRECISION NOT NULL
);

-- Turn snapshots into a hypertable, chunked by day.
SELECT create_hypertable('snapshots', 'time', if_not_exists => TRUE, chunk_time_interval => INTERVAL '1 day');

CREATE INDEX IF NOT EXISTS idx_snapshots_probe_time ON snapshots (probe_id, time DESC);

-- Keep raw per-probe snapshots for 30 days; adjust to taste once volumes are known.
SELECT add_retention_policy('snapshots', INTERVAL '30 days', if_not_exists => TRUE);
