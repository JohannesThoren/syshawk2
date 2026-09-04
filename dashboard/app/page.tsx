"use client";

import { useEffect, useState } from "react";
import { useProbes } from "@/lib/useProbes";
import { useAuth } from "@/lib/useAuth";
import { ServerList } from "@/components/ServerList";
import { ServerDetail } from "@/components/ServerDetail";
import { LoginScreen } from "@/components/LoginScreen";

export default function Home() {
  const { me, loginError, loggingIn, login, logout } = useAuth();
  const authed = !!me;

  if (me === undefined) {
    return (
      <div className="flex h-screen items-center justify-center text-[var(--text-muted)] text-sm">
        Loading…
      </div>
    );
  }

  if (!authed) {
    return <LoginScreen onLogin={login} error={loginError} loading={loggingIn} />;
  }

  return <Dashboard username={me.username} onLogout={logout} />;
}

function Dashboard({
  username,
  onLogout,
}: {
  username: string;
  onLogout: () => void;
}) {
  const { probes, connected, error } = useProbes();
  const [selectedId, setSelectedId] = useState<string | null>(null);

  useEffect(() => {
    if (!selectedId && probes.length > 0) {
      setSelectedId(probes[0].id);
    }
  }, [probes, selectedId]);

  const selected = probes.find((p) => p.id === selectedId) ?? null;

  return (
    <div className="flex h-screen">
      <aside className="w-72 shrink-0 border-r border-[var(--border)] flex flex-col">
        <div className="px-4 py-4 border-b border-[var(--border)] flex items-center justify-between">
          <span className="font-semibold tracking-tight">Shawk</span>
          <span
            className="text-[10px] font-mono px-1.5 py-0.5 rounded"
            style={{
              color: connected ? "var(--status-online)" : "var(--status-offline)",
              background: connected
                ? "var(--status-online-dim)"
                : "var(--status-offline-dim)",
            }}
          >
            {connected ? "LIVE" : "RECONNECTING"}
          </span>
        </div>
        <div className="flex-1 overflow-y-auto">
          <ServerList
            probes={probes}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
        </div>
        <div className="px-4 py-3 border-t border-[var(--border)] flex items-center justify-between">
          <span className="text-xs font-mono text-[var(--text-muted)]">
            {username}
          </span>
          <button
            onClick={onLogout}
            className="text-xs text-[var(--text-muted)] hover:text-[var(--text)]"
          >
            Sign out
          </button>
        </div>
      </aside>

      <main className="flex-1 overflow-y-auto">
        {error ? (
          <div className="flex h-full items-center justify-center text-[var(--status-offline)] text-sm">
            Couldn't reach the server: {error}
          </div>
        ) : selected ? (
          <ServerDetail probe={selected} />
        ) : (
          <div className="flex h-full items-center justify-center text-[var(--text-muted)] text-sm">
            No server selected.
          </div>
        )}
      </main>
    </div>
  );
}
