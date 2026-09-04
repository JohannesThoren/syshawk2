import { MetricChart } from "./MetricChart";

export function MetricCard({
  label,
  value,
  sub,
  history,
  color,
  max,
  onClick,
}: {
  label: string;
  value: string;
  sub?: string;
  history: number[];
  color: string;
  max?: number;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      disabled={!onClick}
      className="text-left rounded-lg border border-[var(--border)] bg-[var(--surface)] p-4 transition-colors enabled:hover:bg-[var(--surface-raised)] enabled:hover:border-[var(--text-faint)] disabled:cursor-default w-full"
    >
      <div className="flex items-baseline justify-between">
        <span className="text-xs text-[var(--text-muted)] uppercase tracking-wide">
          {label}
        </span>
        {sub && (
          <span className="text-xs text-[var(--text-faint)] font-mono">{sub}</span>
        )}
      </div>
      <div className="mt-1 text-2xl font-mono font-semibold tabular text-[var(--text)]">
        {value}
      </div>
      <div className="mt-2 -mx-1">
        <MetricChart data={history} color={color} max={max} />
      </div>
    </button>
  );
}
