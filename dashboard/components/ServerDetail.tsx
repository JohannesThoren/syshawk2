"use client";

import { useEffect, useRef, useState } from "react";
import { ProbeSummary, Snapshot } from "@/lib/types";
import { fetchHistory, formatBytes, formatUptime, timeAgo } from "@/lib/api";
import { StatusDot } from "./StatusDot";
import { MetricCard } from "./MetricCard";
import { ProcessTable } from "./ProcessTable";
import { Terminal } from "./Terminal";
import { MetricDetailModal, MetricKind } from "./MetricDetailModal";

const HISTORY_WINDOW = 120;

export function ServerDetail({ probe }: { probe: ProbeSummary }) {
  const [history, setHistory] = useState<Snapshot[]>([]);
  const [terminalOpen, setTerminalOpen] = useState(false);
  const [openMetric, setOpenMetric] = useState<MetricKind | null>(null);
  const lastTimestamp = useRef<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setHistory([]);
    setTerminalOpen(false);
    setOpenMetric(null);
    lastTimestamp.current = null;
    fetchHistory(probe.id)
      .then((data) => {
        if (cancelled) return;
        setHistory(data.slice(-HISTORY_WINDOW));
        lastTimestamp.current = data.at(-1)?.timestamp ?? null;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [probe.id]);

  useEffect(() => {
    const latest = probe.latest;
    if (!latest || latest.timestamp === lastTimestamp.current) return;
    lastTimestamp.current = latest.timestamp;
    setHistory((prev) => [...prev, latest].slice(-HISTORY_WINDOW));
  }, [probe.latest]);

  const latest = probe.latest;

  if (!latest) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-[var(--text-muted)] gap-2">
        <StatusDot status={probe.status} withLabel />
        <p className="text-sm">
          {probe.status === "pending"
            ? "Waiting for this probe to check in for the first time."
            : "No data available yet."}
        </p>
      </div>
    );
  }

  const memPct = (latest.memory.used_bytes / latest.memory.total_bytes) * 100;
  const rootDisk =
    latest.disks.find((d) => d.mount_point === "/") ?? latest.disks[0];
  const diskPct = rootDisk
    ? ((rootDisk.total_bytes - rootDisk.available_bytes) / rootDisk.total_bytes) * 100
    : 0;
  // Raw deltas are "bytes since the previous snapshot" - only a real
  // bytes/sec rate once divided by the actual elapsed time, since the
  // probe's report interval is configurable (and network jitter means
  // consecutive snapshots are never exactly evenly spaced).
  function throughputBytesPerSec(curr: Snapshot, prev: Snapshot | undefined): number {
    if (!prev) return 0;
    const elapsedSec =
      (new Date(curr.timestamp).getTime() - new Date(prev.timestamp).getTime()) / 1000;
    if (elapsedSec <= 0) return 0;
    const totalDelta = curr.network.reduce(
      (sum, n) => sum + n.bytes_received_delta + n.bytes_transmitted_delta,
      0
    );
    return totalDelta / elapsedSec;
  }

  const netHistory = history.map((s, i) => throughputBytesPerSec(s, history[i - 1]));
  const netTotal = netHistory.at(-1) ?? 0;

  const severityColor = (pct: number) =>
    pct >= 90 ? "#e4573d" : pct >= 75 ? "#d9a441" : "#3ecf8e";

  const cpuHistory = history.map((s) => s.cpu.global_usage_pct);
  const memHistory = history.map(
    (s) => (s.memory.used_bytes / s.memory.total_bytes) * 100
  );
  const diskHistory = history.map((s) => {
    const d = s.disks.find((x) => x.mount_point === "/") ?? s.disks[0];
    return d ? ((d.total_bytes - d.available_bytes) / d.total_bytes) * 100 : 0;
  });

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-semibold">{probe.name}</h1>
            <StatusDot status={probe.status} withLabel />
          </div>
          <p className="mt-1 text-sm text-[var(--text-muted)] font-mono">
            {latest.hostname} · {latest.cpu.brand} · {latest.cpu.core_count} cores
          </p>
        </div>
        <div className="text-right text-sm text-[var(--text-muted)] font-mono">
          <div>uptime {formatUptime(latest.uptime_secs)}</div>
          <div>updated {timeAgo(latest.timestamp)}</div>
        </div>
      </div>

      <button
        onClick={() => setTerminalOpen(true)}
        disabled={probe.status !== "online"}
        className="text-sm px-3 py-1.5 rounded-md border border-[var(--border)] bg-[var(--surface)] hover:bg-[var(--surface-raised)] disabled:opacity-40 disabled:cursor-not-allowed font-mono"
      >
        &gt;_ Open terminal
      </button>

      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
        <MetricCard
          label="CPU"
          value={`${latest.cpu.global_usage_pct.toFixed(1)}%`}
          sub={`load ${latest.load_average.one.toFixed(2)}`}
          history={cpuHistory}
          color={severityColor(latest.cpu.global_usage_pct)}
          max={100}
          onClick={() => setOpenMetric("cpu")}
        />
        <MetricCard
          label="Memory"
          value={`${memPct.toFixed(1)}%`}
          sub={formatBytes(latest.memory.used_bytes)}
          history={memHistory}
          color={severityColor(memPct)}
          max={100}
          onClick={() => setOpenMetric("memory")}
        />
        <MetricCard
          label="Disk (/)"
          value={`${diskPct.toFixed(1)}%`}
          sub={rootDisk ? formatBytes(rootDisk.available_bytes) + " free" : "—"}
          history={diskHistory}
          color={severityColor(diskPct)}
          max={100}
          onClick={() => setOpenMetric("disk")}
        />
        <MetricCard
          label="Network"
          value={formatBytes(netTotal) + "/s"}
          sub={`${latest.network.length} interfaces`}
          history={netHistory}
          color="#c17ee8"
          onClick={() => setOpenMetric("network")}
        />
      </div>

      {openMetric && (
        <MetricDetailModal
          metric={openMetric}
          history={history}
          onClose={() => setOpenMetric(null)}
        />
      )}
      <div>
        <h2 className="text-sm font-medium text-[var(--text-muted)] mb-2">
          Top processes
        </h2>
        <ProcessTable processes={latest.processes} />
      </div>

      <div>
        <h2 className="text-sm font-medium text-[var(--text-muted)] mb-2">Disks</h2>
        <div className="rounded-lg border border-[var(--border)] bg-[var(--surface)] divide-y divide-[var(--border)]">
          {latest.disks.map((d) => {
            const used = d.total_bytes - d.available_bytes;
            const pct = d.total_bytes > 0 ? (used / d.total_bytes) * 100 : 0;
            return (
              <div key={d.mount_point} className="px-4 py-2.5 flex items-center gap-4">
                <div className="w-40 truncate text-sm font-mono">{d.mount_point}</div>
                <div className="flex-1">
                  <div className="h-1.5 w-full rounded-full bg-[var(--border)] overflow-hidden">
                    <div
                      className="h-full rounded-full bg-[var(--status-pending)]"
                      style={{ width: `${Math.min(100, pct)}%` }}
                    />
                  </div>
                </div>
                <div className="w-40 text-right text-xs font-mono text-[var(--text-muted)] tabular">
                  {formatBytes(used)} / {formatBytes(d.total_bytes)}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {terminalOpen && (
        <Terminal probeId={probe.id} onClose={() => setTerminalOpen(false)} />
      )}
    </div>
  );
}
