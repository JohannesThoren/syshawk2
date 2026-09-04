use chrono::{DateTime, Utc};
use serde_json::Value as Json;
use shawk_common::{
    CpuInfo, DiskInfo, LoadAverage, MemoryInfo, NetworkInfo, ProcessInfo, Snapshot,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct ProbeRow {
    pub id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SnapshotRow {
    pub time: DateTime<Utc>,
    pub probe_id: Uuid,
    pub hostname: String,
    pub uptime_secs: i64,
    pub cpu_global_pct: f32,
    pub cpu_per_core_pct: Json,
    pub cpu_core_count: i32,
    pub cpu_brand: String,
    pub mem_total_bytes: i64,
    pub mem_used_bytes: i64,
    pub swap_total_bytes: i64,
    pub swap_used_bytes: i64,
    pub disks: Json,
    pub network: Json,
    pub processes: Json,
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
}

impl SnapshotRow {
    pub fn into_snapshot(self) -> anyhow::Result<Snapshot> {
        Ok(Snapshot {
            hostname: self.hostname,
            timestamp: self.time,
            uptime_secs: self.uptime_secs as u64,
            cpu: CpuInfo {
                global_usage_pct: self.cpu_global_pct,
                per_core_usage_pct: serde_json::from_value(self.cpu_per_core_pct)?,
                core_count: self.cpu_core_count as usize,
                brand: self.cpu_brand,
            },
            memory: MemoryInfo {
                total_bytes: self.mem_total_bytes as u64,
                used_bytes: self.mem_used_bytes as u64,
                swap_total_bytes: self.swap_total_bytes as u64,
                swap_used_bytes: self.swap_used_bytes as u64,
            },
            disks: serde_json::from_value::<Vec<DiskInfo>>(self.disks)?,
            network: serde_json::from_value::<Vec<NetworkInfo>>(self.network)?,
            processes: serde_json::from_value::<Vec<ProcessInfo>>(self.processes)?,
            load_average: LoadAverage {
                one: self.load1,
                five: self.load5,
                fifteen: self.load15,
            },
        })
    }
}

pub async fn find_probe_by_token_hash(pool: &PgPool, token_hash: &str) -> sqlx::Result<Option<ProbeRow>> {
    sqlx::query_as::<_, ProbeRow>("SELECT * FROM probes WHERE token_hash = $1")
        .bind(token_hash)
        .fetch_optional(pool)
        .await
}

pub async fn create_probe(pool: &PgPool, name: &str, token_hash: &str) -> sqlx::Result<ProbeRow> {
    sqlx::query_as::<_, ProbeRow>(
        "INSERT INTO probes (name, token_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(name)
    .bind(token_hash)
    .fetch_one(pool)
    .await
}

pub async fn list_probes(pool: &PgPool) -> sqlx::Result<Vec<ProbeRow>> {
    sqlx::query_as::<_, ProbeRow>("SELECT * FROM probes ORDER BY name")
        .fetch_all(pool)
        .await
}

pub async fn touch_last_seen(pool: &PgPool, probe_id: Uuid, at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("UPDATE probes SET last_seen = $1 WHERE id = $2")
        .bind(at)
        .bind(probe_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_snapshot(pool: &PgPool, probe_id: Uuid, s: &Snapshot) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO snapshots (
            time, probe_id, hostname, uptime_secs,
            cpu_global_pct, cpu_per_core_pct, cpu_core_count, cpu_brand,
            mem_total_bytes, mem_used_bytes, swap_total_bytes, swap_used_bytes,
            disks, network, processes, load1, load5, load15
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
        "#,
    )
    .bind(s.timestamp)
    .bind(probe_id)
    .bind(&s.hostname)
    .bind(s.uptime_secs as i64)
    .bind(s.cpu.global_usage_pct)
    .bind(serde_json::to_value(&s.cpu.per_core_usage_pct).unwrap())
    .bind(s.cpu.core_count as i32)
    .bind(&s.cpu.brand)
    .bind(s.memory.total_bytes as i64)
    .bind(s.memory.used_bytes as i64)
    .bind(s.memory.swap_total_bytes as i64)
    .bind(s.memory.swap_used_bytes as i64)
    .bind(serde_json::to_value(&s.disks).unwrap())
    .bind(serde_json::to_value(&s.network).unwrap())
    .bind(serde_json::to_value(&s.processes).unwrap())
    .bind(s.load_average.one)
    .bind(s.load_average.five)
    .bind(s.load_average.fifteen)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_snapshot(pool: &PgPool, probe_id: Uuid) -> sqlx::Result<Option<SnapshotRow>> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT * FROM snapshots WHERE probe_id = $1 ORDER BY time DESC LIMIT 1",
    )
    .bind(probe_id)
    .fetch_optional(pool)
    .await
}

pub async fn history(
    pool: &PgPool,
    probe_id: Uuid,
    since: DateTime<Utc>,
) -> sqlx::Result<Vec<SnapshotRow>> {
    sqlx::query_as::<_, SnapshotRow>(
        "SELECT * FROM snapshots WHERE probe_id = $1 AND time >= $2 ORDER BY time ASC",
    )
    .bind(probe_id)
    .bind(since)
    .fetch_all(pool)
    .await
}
