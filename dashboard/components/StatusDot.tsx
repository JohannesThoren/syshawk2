import { ProbeStatus } from "@/lib/types";

const LABEL: Record<ProbeStatus, string> = {
  online: "Online",
  offline: "Offline",
  pending: "Awaiting first check-in",
};

const COLOR: Record<ProbeStatus, string> = {
  online: "var(--status-online)",
  offline: "var(--status-offline)",
  pending: "var(--status-pending)",
};

export function StatusDot({
  status,
  withLabel = false,
}: {
  status: ProbeStatus;
  withLabel?: boolean;
}) {
  return (
    <span className="inline-flex items-center gap-2">
      <span
        className="inline-block h-2 w-2 rounded-full shrink-0"
        style={{
          background: COLOR[status],
          boxShadow: status === "online" ? `0 0 6px ${COLOR[status]}` : "none",
        }}
      />
      {withLabel && (
        <span className="text-sm" style={{ color: COLOR[status] }}>
          {LABEL[status]}
        </span>
      )}
    </span>
  );
}
