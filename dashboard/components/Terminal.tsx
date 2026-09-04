"use client";

import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { apiBase } from "@/lib/api";

export function Terminal({ probeId, onClose }: { probeId: string; onClose: () => void }) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new XTerm({
      convertEol: true,
      fontFamily: "var(--font-mono), monospace",
      fontSize: 13,
      theme: {
        background: "#0f1319",
        foreground: "#e6ecf3",
        cursor: "#3ecf8e",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();

    const wsBase = apiBase().replace(/^http/, "ws");
    const ws = new WebSocket(`${wsBase}/api/probes/${probeId}/terminal`);
    ws.binaryType = "arraybuffer";

    ws.onopen = () => term.writeln("connected.\r\n");
    ws.onmessage = (evt) => {
      if (typeof evt.data === "string") {
        term.write(evt.data);
      } else {
        term.write(new Uint8Array(evt.data));
      }
    };
    ws.onclose = () => term.writeln("\r\n\r\n[session closed]");
    ws.onerror = () => term.writeln("\r\n\r\n[connection error]");

    term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(new TextEncoder().encode(data));
      }
    });

    const resizeObserver = new ResizeObserver(() => {
      fit.fit();
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(
          JSON.stringify({ type: "resize", cols: term.cols, rows: term.rows })
        );
      }
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      ws.close();
      term.dispose();
    };
  }, [probeId]);

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-black/70 backdrop-blur-sm">
      <div className="flex items-center justify-between px-4 py-2 bg-[var(--surface)] border-b border-[var(--border)]">
        <span className="text-sm font-mono text-[var(--text-muted)]">
          terminal
        </span>
        <button
          onClick={onClose}
          className="text-xs px-2 py-1 rounded border border-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text)]"
        >
          Close
        </button>
      </div>
      <div className="flex-1 p-3 bg-[#0f1319] overflow-hidden" ref={containerRef} />
    </div>
  );
}
