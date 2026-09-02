export function UsageBar({
  pct,
  warnAt = 75,
  dangerAt = 90,
}: {
  pct: number;
  warnAt?: number;
  dangerAt?: number;
}) {
  const clamped = Math.max(0, Math.min(100, pct));
  const color =
    clamped >= dangerAt
      ? "var(--status-offline)"
      : clamped >= warnAt
      ? "var(--status-pending)"
      : "var(--status-online)";

  return (
    <div className="h-1.5 w-full rounded-full bg-[var(--border)] overflow-hidden">
      <div
        className="h-full rounded-full transition-[width] duration-500 ease-out"
        style={{ width: `${clamped}%`, background: color }}
      />
    </div>
  );
}
