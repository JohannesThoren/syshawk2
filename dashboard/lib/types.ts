export type ProbeStatus = "online" | "offline" | "pending";

export interface CpuInfo {
  global_usage_pct: number;
  per_core_usage_pct: number[];
  core_count: number;
  brand: string;
}

export interface MemoryInfo {
  total_bytes: number;
  used_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
}

export interface DiskInfo {
  mount_point: string;
  name: string;
  total_bytes: number;
  available_bytes: number;
  file_system: string;
  is_removable: boolean;
}

export interface NetworkInfo {
  interface_name: string;
  bytes_received_total: number;
  bytes_transmitted_total: number;
  bytes_received_delta: number;
  bytes_transmitted_delta: number;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage_pct: number;
  memory_bytes: number;
  status: string;
}

export interface LoadAverage {
  one: number;
  five: number;
  fifteen: number;
}

export interface Snapshot {
  hostname: string;
  timestamp: string;
  uptime_secs: number;
  cpu: CpuInfo;
  memory: MemoryInfo;
  disks: DiskInfo[];
  network: NetworkInfo[];
  processes: ProcessInfo[];
  load_average: LoadAverage;
}

export interface ProbeSummary {
  id: string;
  name: string;
  hostname: string | null;
  status: ProbeStatus;
  last_seen: string | null;
  latest: Snapshot | null;
}

export type WsEvent =
  | { type: "snapshot"; probe_id: string; snapshot: Snapshot }
  | { type: "status_changed"; probe_id: string; status: ProbeStatus };
