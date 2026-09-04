import { ProcessInfo } from "@/lib/types";
import { formatBytes } from "@/lib/api";

export function ProcessTable({ processes }: { processes: ProcessInfo[] }) {
  return (
    <div className="rounded-lg border border-[var(--border)] bg-[var(--surface)] overflow-hidden">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-[var(--border)] text-left text-[var(--text-faint)]">
            <th className="font-normal px-4 py-2.5 text-xs uppercase tracking-wide">
              Process
            </th>
            <th className="font-normal px-4 py-2.5 text-xs uppercase tracking-wide w-20">
              PID
            </th>
            <th className="font-normal px-4 py-2.5 text-xs uppercase tracking-wide w-20 text-right">
              CPU
            </th>
            <th className="font-normal px-4 py-2.5 text-xs uppercase tracking-wide w-24 text-right">
              Memory
            </th>
          </tr>
        </thead>
        <tbody className="font-mono">
          {processes.map((p) => (
            <tr
              key={p.pid}
              className="border-b border-[var(--border)] last:border-0"
            >
              <td className="px-4 py-2 truncate max-w-0 text-[var(--text)]">
                {p.name}
              </td>
              <td className="px-4 py-2 text-[var(--text-muted)] tabular">
                {p.pid}
              </td>
              <td className="px-4 py-2 text-right tabular text-[var(--text)]">
                {p.cpu_usage_pct.toFixed(1)}%
              </td>
              <td className="px-4 py-2 text-right tabular text-[var(--text-muted)]">
                {formatBytes(p.memory_bytes)}
              </td>
            </tr>
          ))}
          {processes.length === 0 && (
            <tr>
              <td
                colSpan={4}
                className="px-4 py-6 text-center text-[var(--text-muted)] font-sans"
              >
                No process data yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
