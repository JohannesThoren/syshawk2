use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single point-in-time snapshot of a monitored host, sent by the probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub hostname: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_secs: u64,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
    pub processes: Vec<ProcessInfo>,
    pub load_average: LoadAverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub global_usage_pct: f32,
    pub per_core_usage_pct: Vec<f32>,
    pub core_count: usize,
    pub brand: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub name: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub file_system: String,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface_name: String,
    pub bytes_received_total: u64,
    pub bytes_transmitted_total: u64,
    /// Delta since the previous snapshot, used for computing throughput.
    pub bytes_received_delta: u64,
    pub bytes_transmitted_delta: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_pct: f32,
    pub memory_bytes: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

/// Body the probe POSTs to the server on every collection interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub snapshot: Snapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResponse {
    pub accepted: bool,
}

/// Server-side view of a registered probe, returned to the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSummary {
    pub id: Uuid,
    pub name: String,
    pub hostname: Option<String>,
    pub status: ProbeStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub latest: Option<Snapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    Online,
    Offline,
    /// Registered but has never sent a snapshot.
    Pending,
}

/// Request body for registering a new probe from the dashboard/admin API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterProbeRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterProbeResponse {
    pub id: Uuid,
    pub name: String,
    /// Plaintext token - shown once, never retrievable again.
    pub token: String,
}

/// Silence threshold, in seconds, after which a probe is considered offline.
pub const OFFLINE_THRESHOLD_SECS: i64 = 30;

/// Messages broadcast to dashboard clients over the WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Snapshot { probe_id: Uuid, snapshot: Snapshot },
    StatusChanged { probe_id: Uuid, status: ProbeStatus },
}

/// Sent server->probe over the probe's persistent control connection to ask
/// it to open a dedicated terminal socket for an interactive shell session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProbeControlMsg {
    OpenTerminal {
        session_id: Uuid,
        cols: u16,
        rows: u16,
    },
}

/// A control frame sent on an already-open terminal socket (in either
/// direction) to resize the PTY. Raw PTY bytes travel as Binary websocket
/// frames on the same socket and are never wrapped in this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TerminalControlMsg {
    Resize { cols: u16, rows: u16 },
}
