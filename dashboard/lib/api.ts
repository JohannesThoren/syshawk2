import { ProbeSummary, Snapshot } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

export function apiBase() {
  return API_URL;
}

export function wsUrl() {
  return API_URL.replace(/^http/, "ws") + "/api/ws";
}

export async function fetchProbes(): Promise<ProbeSummary[]> {
  const res = await fetch(`${API_URL}/api/probes`, {
    cache: "no-store",
    credentials: "include",
  });
  if (!res.ok) throw new Error(`failed to fetch probes: ${res.status}`);
  return res.json();
}

export async function fetchHistory(probeId: string): Promise<Snapshot[]> {
  const res = await fetch(`${API_URL}/api/probes/${probeId}/history`, {
    cache: "no-store",
    credentials: "include",
  });
  if (!res.ok) throw new Error(`failed to fetch history: ${res.status}`);
  return res.json();
}

export interface Me {
  username: string;
}

export async function fetchMe(): Promise<Me | null> {
  const res = await fetch(`${API_URL}/api/auth/me`, {
    cache: "no-store",
    credentials: "include",
  });
  if (res.status === 401) return null;
  if (!res.ok) throw new Error(`failed to check session: ${res.status}`);
  return res.json();
}

export async function login(username: string, password: string): Promise<Me> {
  const res = await fetch(`${API_URL}/api/auth/login`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  if (!res.ok) {
    throw new Error("Incorrect username/password, or not authorized.");
  }
  return res.json();
}

export async function logout(): Promise<void> {
  await fetch(`${API_URL}/api/auth/logout`, {
    method: "POST",
    credentials: "include",
  });
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1
  );
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatUptime(secs: number): string {
  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

export function timeAgo(iso: string | null): string {
  if (!iso) return "never";
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 5) return "just now";
  if (diff < 60) return `${Math.floor(diff)}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}
