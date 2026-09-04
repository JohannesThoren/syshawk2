"use client";

import { Snapshot } from "@/lib/types";
import { formatBytes } from "@/lib/api";
import { DetailChart } from "./DetailChart";
import { UsageBar } from "./UsageBar";

export type MetricKind = "cpu" | "memory" | "disk" | "network";

const TITLES: Record<MetricKind, string> = {
  cpu: "CPU",
  memory: "Memory",
  disk: "Disk",
  network: "Network",
};

function pctColor(pct: number) {
  return pct >= 90 ? "#e4573d" : pct >= 75 ? "#d9a441" : "#3ecf8e";
}

function throughput(curr: Snapshot, prev: Snapshot | undefined, pick: (n: Snapshot["network"][number]) => number) {
  if (!prev) return 0;
  const elapsedSec = (new Date(curr.timestamp).getTime() - new Date(prev.timestamp).getTime()) / 1000;
  if (elapsedSec <= 0) return 0;
  const total = curr.network.reduce((sum, n) => sum + pick(n), 0);
  return total / elapsedSec;
}

export function MetricDetailModal({
  metric,
  history,
  onClose,
}: {
  metric: MetricKind;
  history: Snapshot[];
  onClose: () => void;
}) {
  const latest = history.at(-1);
  if (!latest) return null;
  const timestamps = history.map((s) => s.timestamp);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-6"
      onClick={onClose}
    >
      <div
        className="w-full max-w-3xl max-h-[85vh] overflow-y-auto rounded-lg border border-[var(--border)] bg-[var(--surface)]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-[var(--border)]">
          <h2 className="text-sm font-medium">{TITLES[metric]}</h2>
          <button
            onClick={onClose}
            className="text-xs px-2 py-1 rounded border border-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text)]"
          >
            Close
          </button>
        </div>
        <div className="p-5">
          {metric === "cpu" && <CpuDetail history={history} latest={latest} timestamps={timestamps} />}
          {metric === "memory" && <MemoryDetail history={history} latest={latest} timestamps={timestamps} />}
          {metric === "disk" && <DiskDetail history={history} latest={latest} timestamps={timestamps} />}
          {metric === "network" && <NetworkDetail history={history} latest={latest} timestamps={timestamps} />}
        </div>
      </div>
    </div>
  );
}

function StatBox({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-[var(--border)] bg-[var(--surface-raised)] px-3 py-2">
      <div className="text-[10px] uppercase tracking-wide text-[var(--text-faint)]">{label}</div>
      <div className="mt-0.5 font-mono text-sm text-[var(--text)] tabular">{value}</div>
    </div>
  );
}

function CpuDetail({
  history,
  latest,
  timestamps,
}: {
  history: Snapshot[];
  latest: Snapshot;
  timestamps: string[];
}) {
  const usageHistory = history.map((s) => s.cpu.global_usage_pct);

  return (
    <div className="space-y-5">
      <p className="text-xs text-[var(--text-muted)] font-mono">
        {latest.cpu.brand} · {latest.cpu.core_count} cores
      </p>

      <DetailChart
        series={[{ name: "Usage %", color: pctColor(latest.cpu.global_usage_pct), data: usageHistory }]}
        timestamps={timestamps}
        max={100}
        valueFormatter={(v) => `${v.toFixed(0)}%`}
      />

      <div className="grid grid-cols-3 gap-3">
        <StatBox label="Load (1m)" value={latest.load_average.one.toFixed(2)} />
        <StatBox label="Load (5m)" value={latest.load_average.five.toFixed(2)} />
        <StatBox label="Load (15m)" value={latest.load_average.fifteen.toFixed(2)} />
      </div>

      <div>
        <h3 className="text-xs text-[var(--text-muted)] mb-2">Per core</h3>
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-x-4 gap-y-2">
          {latest.cpu.per_core_usage_pct.map((pct, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="text-[10px] w-10 text-[var(--text-faint)] font-mono">
                #{i}
              </span>
              <UsageBar pct={pct} />
              <span className="text-[10px] w-9 text-right text-[var(--text-muted)] font-mono tabular">
                {pct.toFixed(0)}%
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function MemoryDetail({
  history,
  latest,
  timestamps,
}: {
  history: Snapshot[];
  latest: Snapshot;
  timestamps: string[];
}) {
  const pctHistory = history.map((s) => (s.memory.used_bytes / s.memory.total_bytes) * 100);
  const pct = (latest.memory.used_bytes / latest.memory.total_bytes) * 100;
  const swapPct =
    latest.memory.swap_total_bytes > 0
      ? (latest.memory.swap_used_bytes / latest.memory.swap_total_bytes) * 100
      : 0;

  return (
    <div className="space-y-5">
      <DetailChart
        series={[{ name: "Used %", color: pctColor(pct), data: pctHistory }]}
        timestamps={timestamps}
        max={100}
        valueFormatter={(v) => `${v.toFixed(0)}%`}
      />

      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <StatBox label="Used" value={formatBytes(latest.memory.used_bytes)} />
        <StatBox label="Total" value={formatBytes(latest.memory.total_bytes)} />
        <StatBox
          label="Free"
          value={formatBytes(latest.memory.total_bytes - latest.memory.used_bytes)}
        />
        <StatBox
          label="Swap"
          value={
            latest.memory.swap_total_bytes > 0
              ? `${formatBytes(latest.memory.swap_used_bytes)} / ${formatBytes(latest.memory.swap_total_bytes)}`
              : "none"
          }
        />
      </div>

      {latest.memory.swap_total_bytes > 0 && (
        <div>
          <h3 className="text-xs text-[var(--text-muted)] mb-2">Swap usage</h3>
          <UsageBar pct={swapPct} />
        </div>
      )}
    </div>
  );
}

function DiskDetail({
  history,
  latest,
  timestamps,
}: {
  history: Snapshot[];
  latest: Snapshot;
  timestamps: string[];
}) {
  const rootDisk = latest.disks.find((d) => d.mount_point === "/") ?? latest.disks[0];
  const pctHistory = history.map((s) => {
    const d = s.disks.find((x) => x.mount_point === rootDisk?.mount_point) ?? s.disks[0];
    return d ? ((d.total_bytes - d.available_bytes) / d.total_bytes) * 100 : 0;
  });
  const rootPct = rootDisk
    ? ((rootDisk.total_bytes - rootDisk.available_bytes) / rootDisk.total_bytes) * 100
    : 0;

  return (
    <div className="space-y-5">
      <p className="text-xs text-[var(--text-muted)] font-mono">
        {rootDisk?.mount_point} · {rootDisk?.file_system}
      </p>

      <DetailChart
        series={[{ name: "Used %", color: pctColor(rootPct), data: pctHistory }]}
        timestamps={timestamps}
        max={100}
        valueFormatter={(v) => `${v.toFixed(0)}%`}
      />

      <div>
        <h3 className="text-xs text-[var(--text-muted)] mb-2">All disks</h3>
        <div className="rounded-lg border border-[var(--border)] divide-y divide-[var(--border)]">
          {latest.disks.map((d) => {
            const used = d.total_bytes - d.available_bytes;
            const pct = d.total_bytes > 0 ? (used / d.total_bytes) * 100 : 0;
            return (
              <div key={d.mount_point} className="px-3 py-2.5 flex items-center gap-4">
                <div className="w-36 truncate text-sm font-mono">{d.mount_point}</div>
                <div className="flex-1">
                  <UsageBar pct={pct} />
                </div>
                <div className="w-36 text-right text-xs font-mono text-[var(--text-muted)] tabular">
                  {formatBytes(used)} / {formatBytes(d.total_bytes)}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function NetworkDetail({
  history,
  latest,
  timestamps,
}: {
  history: Snapshot[];
  latest: Snapshot;
  timestamps: string[];
}) {
  const rxHistory = history.map((s, i) =>
    throughput(s, history[i - 1], (n) => n.bytes_received_delta)
  );
  const txHistory = history.map((s, i) =>
    throughput(s, history[i - 1], (n) => n.bytes_transmitted_delta)
  );
  const prev = history[history.length - 2];

  return (
    <div className="space-y-5">
      <DetailChart
        series={[
          { name: "Received", color: "#5b8def", data: rxHistory },
          { name: "Sent", color: "#c17ee8", data: txHistory },
        ]}
        timestamps={timestamps}
        valueFormatter={(v) => `${formatBytes(v)}/s`}
      />

      <div>
        <h3 className="text-xs text-[var(--text-muted)] mb-2">Interfaces</h3>
        <div className="rounded-lg border border-[var(--border)] divide-y divide-[var(--border)]">
          <div className="px-3 py-2 flex items-center gap-4 text-[10px] uppercase tracking-wide text-[var(--text-faint)]">
            <div className="w-28">Interface</div>
            <div className="flex-1 text-right">Down</div>
            <div className="flex-1 text-right">Up</div>
            <div className="flex-1 text-right">Total</div>
          </div>
          {latest.network.map((n) => {
            const elapsedSec = prev
              ? (new Date(latest.timestamp).getTime() - new Date(prev.timestamp).getTime()) / 1000
              : 0;
            const rxRate = elapsedSec > 0 ? n.bytes_received_delta / elapsedSec : 0;
            const txRate = elapsedSec > 0 ? n.bytes_transmitted_delta / elapsedSec : 0;
            return (
              <div key={n.interface_name} className="px-3 py-2 flex items-center gap-4 text-sm font-mono">
                <div className="w-28 truncate">{n.interface_name}</div>
                <div className="flex-1 text-right text-[var(--text-muted)]">
                  {formatBytes(rxRate)}/s
                </div>
                <div className="flex-1 text-right text-[var(--text-muted)]">
                  {formatBytes(txRate)}/s
                </div>
                <div className="flex-1 text-right text-[var(--text-faint)] text-xs">
                  {formatBytes(n.bytes_received_total + n.bytes_transmitted_total)}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
