"use client";

import { useEffect, useRef, useState } from "react";
import { fetchProbes, wsUrl } from "./api";
import { ProbeSummary, WsEvent } from "./types";

export function useProbes() {
  const [probes, setProbes] = useState<ProbeSummary[]>([]);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const probesRef = useRef<ProbeSummary[]>([]);

  useEffect(() => {
    let cancelled = false;
    let ws: WebSocket | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout>;
    let pollTimer: ReturnType<typeof setInterval>;

    async function loadOnce() {
      try {
        const data = await fetchProbes();
        if (cancelled) return;
        probesRef.current = data;
        setProbes(data);
        setError(null);
      } catch (e) {
        if (!cancelled) setError((e as Error).message);
      }
    }

    function connect() {
      ws = new WebSocket(wsUrl());
      ws.onopen = () => setConnected(true);
      ws.onclose = () => {
        setConnected(false);
        reconnectTimer = setTimeout(connect, 2000);
      };
      ws.onerror = () => ws?.close();
      ws.onmessage = (msg) => {
        try {
          const event: WsEvent = JSON.parse(msg.data);
          const current = [...probesRef.current];
          const idx = current.findIndex((p) => p.id === event.probe_id);
          if (idx === -1) return;

          if (event.type === "snapshot") {
            current[idx] = {
              ...current[idx],
              hostname: event.snapshot.hostname,
              status: "online",
              last_seen: event.snapshot.timestamp,
              latest: event.snapshot,
            };
          } else if (event.type === "status_changed") {
            current[idx] = { ...current[idx], status: event.status };
          }
          probesRef.current = current;
          setProbes(current);
        } catch {
          // ignore malformed frames
        }
      };
    }

    loadOnce();
    connect();
    // Fallback refresh in case a WS message is ever missed.
    pollTimer = setInterval(loadOnce, 15000);

    return () => {
      cancelled = true;
      clearTimeout(reconnectTimer);
      clearInterval(pollTimer);
      ws?.close();
    };
  }, []);

  return { probes, connected, error };
}
