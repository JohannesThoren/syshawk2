use chrono::Utc;
use shawk_common::{CpuInfo, DiskInfo, LoadAverage, MemoryInfo, NetworkInfo, ProcessInfo, Snapshot};
use std::collections::HashMap;
use sysinfo::{Disks, Networks, System};

/// Keeps sysinfo's stateful handles alive across collections so that
/// per-tick deltas (CPU %, network throughput) are meaningful.
pub struct Collector {
    system: System,
    disks: Disks,
    networks: Networks,
    prev_net_totals: HashMap<String, (u64, u64)>,
    top_processes: usize,
}

impl Collector {
    pub fn new(top_processes: usize) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            prev_net_totals: HashMap::new(),
            top_processes,
        }
    }

    pub fn collect(&mut self) -> Snapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        self.disks.refresh();
        self.networks.refresh();

        let cpus = self.system.cpus();
        let per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let global_usage = if per_core.is_empty() {
            0.0
        } else {
            per_core.iter().sum::<f32>() / per_core.len() as f32
        };
        let brand = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();

        let cpu = CpuInfo {
            global_usage_pct: global_usage,
            per_core_usage_pct: per_core,
            core_count: cpus.len(),
            brand,
        };

        let memory = MemoryInfo {
            total_bytes: self.system.total_memory(),
            used_bytes: self.system.used_memory(),
            swap_total_bytes: self.system.total_swap(),
            swap_used_bytes: self.system.used_swap(),
        };

        let disks = self
            .disks
            .iter()
            .map(|d| DiskInfo {
                mount_point: d.mount_point().to_string_lossy().to_string(),
                name: d.name().to_string_lossy().to_string(),
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                file_system: d.file_system().to_string_lossy().to_string(),
                is_removable: d.is_removable(),
            })
            .collect();

        let mut network = Vec::new();
        for (name, data) in self.networks.iter() {
            let total_rx = data.total_received();
            let total_tx = data.total_transmitted();
            let (prev_rx, prev_tx) = self
                .prev_net_totals
                .get(name)
                .copied()
                .unwrap_or((total_rx, total_tx));
            network.push(NetworkInfo {
                interface_name: name.clone(),
                bytes_received_total: total_rx,
                bytes_transmitted_total: total_tx,
                bytes_received_delta: total_rx.saturating_sub(prev_rx),
                bytes_transmitted_delta: total_tx.saturating_sub(prev_tx),
            });
            self.prev_net_totals.insert(name.clone(), (total_rx, total_tx));
        }

        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .values()
            .map(|p| ProcessInfo {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage_pct: p.cpu_usage(),
                memory_bytes: p.memory(),
                status: p.status().to_string(),
            })
            .collect();
        processes.sort_by(|a, b| b.cpu_usage_pct.partial_cmp(&a.cpu_usage_pct).unwrap());
        processes.truncate(self.top_processes);

        let load = System::load_average();

        Snapshot {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            timestamp: Utc::now(),
            uptime_secs: System::uptime(),
            cpu,
            memory,
            disks,
            network,
            processes,
            load_average: LoadAverage {
                one: load.one,
                five: load.five,
                fifteen: load.fifteen,
            },
        }
    }
}
