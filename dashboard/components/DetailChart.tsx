"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";

export interface DetailSeries {
  name: string;
  color: string;
  data: number[];
}

export function DetailChart({
  series,
  timestamps,
  max,
  valueFormatter,
}: {
  series: DetailSeries[];
  timestamps: string[];
  max?: number;
  valueFormatter?: (v: number) => string;
}) {
  const points = timestamps.map((t, i) => {
    const point: Record<string, number | string> = {
      t: new Date(t).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" }),
    };
    series.forEach((s) => {
      point[s.name] = s.data[i] ?? 0;
    });
    return point;
  });

  return (
    <ResponsiveContainer width="100%" height={260}>
      <LineChart data={points} margin={{ top: 8, right: 12, bottom: 0, left: 0 }}>
        <CartesianGrid stroke="var(--border)" strokeDasharray="3 3" vertical={false} />
        <XAxis
          dataKey="t"
          tick={{ fill: "var(--text-faint)", fontSize: 11 }}
          axisLine={{ stroke: "var(--border)" }}
          tickLine={false}
          minTickGap={40}
        />
        <YAxis
          domain={[0, max ?? "auto"]}
          tick={{ fill: "var(--text-faint)", fontSize: 11 }}
          axisLine={false}
          tickLine={false}
          width={48}
          tickFormatter={valueFormatter}
        />
        <Tooltip
          contentStyle={{
            background: "var(--surface-raised)",
            border: "1px solid var(--border)",
            borderRadius: 8,
            fontSize: 12,
            fontFamily: "var(--font-mono)",
          }}
          labelStyle={{ color: "var(--text-muted)" }}
          formatter={(value) =>
            valueFormatter ? valueFormatter(Number(value)) : value
          }
        />
        {series.length > 1 && (
          <Legend wrapperStyle={{ fontSize: 12, fontFamily: "var(--font-mono)" }} />
        )}
        {series.map((s) => (
          <Line
            key={s.name}
            type="monotone"
            dataKey={s.name}
            stroke={s.color}
            strokeWidth={1.75}
            dot={false}
            isAnimationActive={false}
          />
        ))}
      </LineChart>
    </ResponsiveContainer>
  );
}
