"use client";

import { ProbeSummary } from "@/lib/types";
import { StatusDot } from "./StatusDot";
import { UsageBar } from "./UsageBar";
import { timeAgo } from "@/lib/api";

export function ServerList({
  probes,
  selectedId,
  onSelect,
}: {
  probes: ProbeSummary[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  const sorted = [...probes].sort((a, b) => a.name.localeCompare(b.name));

  return (
    <div className="flex flex-col">
      {sorted.map((p) => {
        const cpu = p.latest?.cpu.global_usage_pct ?? null;
        const mem = p.latest
          ? (p.latest.memory.used_bytes / p.latest.memory.total_bytes) * 100
          : null;
        const active = p.id === selectedId;

        return (
          <button
            key={p.id}
            onClick={() => onSelect(p.id)}
            className="text-left px-4 py-3 border-b border-[var(--border)] transition-colors"
            style={{
              background: active ? "var(--surface-raised)" : "transparent",
            }}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium text-[var(--text)] truncate">
                {p.name}
              </span>
              <StatusDot status={p.status} />
            </div>
            <div className="mt-1 text-xs text-[var(--text-muted)] font-mono tabular">
              {p.status === "offline"
                ? `last seen ${timeAgo(p.last_seen)}`
                : p.hostname ?? "—"}
            </div>

            {cpu !== null && mem !== null && (
              <div className="mt-2.5 space-y-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] w-8 text-[var(--text-faint)] font-mono">
                    CPU
                  </span>
                  <UsageBar pct={cpu} />
                  <span className="text-[10px] w-9 text-right text-[var(--text-muted)] font-mono tabular">
                    {cpu.toFixed(0)}%
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] w-8 text-[var(--text-faint)] font-mono">
                    MEM
                  </span>
                  <UsageBar pct={mem} />
                  <span className="text-[10px] w-9 text-right text-[var(--text-muted)] font-mono tabular">
                    {mem.toFixed(0)}%
                  </span>
                </div>
              </div>
            )}
          </button>
        );
      })}
      {sorted.length === 0 && (
        <div className="px-4 py-8 text-sm text-[var(--text-muted)]">
          No probes registered yet.
        </div>
      )}
    </div>
  );
}
