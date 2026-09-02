"use client";

import { FormEvent, useState } from "react";

export function LoginScreen({
  onLogin,
  error,
  loading,
}: {
  onLogin: (username: string, password: string) => void;
  error: string | null;
  loading: boolean;
}) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    onLogin(username, password);
  }

  return (
    <div className="flex h-screen items-center justify-center">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-xs rounded-lg border border-[var(--border)] bg-[var(--surface)] p-6"
      >
        <h1 className="text-lg font-semibold mb-1">Shawk</h1>
        <p className="text-sm text-[var(--text-muted)] mb-5">
          Sign in with your server account.
        </p>

        <label className="block text-xs text-[var(--text-muted)] mb-1">
          Username
        </label>
        <input
          autoFocus
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className="w-full mb-3 rounded-md bg-[var(--surface-raised)] border border-[var(--border)] px-3 py-2 text-sm font-mono outline-none focus:border-[var(--text-muted)]"
        />

        <label className="block text-xs text-[var(--text-muted)] mb-1">
          Password
        </label>
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          className="w-full mb-4 rounded-md bg-[var(--surface-raised)] border border-[var(--border)] px-3 py-2 text-sm font-mono outline-none focus:border-[var(--text-muted)]"
        />

        {error && (
          <p className="text-xs text-[var(--status-offline)] mb-3">{error}</p>
        )}

        <button
          type="submit"
          disabled={loading || !username || !password}
          className="w-full rounded-md bg-[var(--status-online)] text-[#08110c] font-medium text-sm py-2 disabled:opacity-40 transition-opacity"
        >
          {loading ? "Signing in…" : "Sign in"}
        </button>
      </form>
    </div>
  );
}
