"use client";

import { AreaChart, Area, ResponsiveContainer, YAxis } from "recharts";

export function MetricChart({
  data,
  color,
  max,
}: {
  data: number[];
  color: string;
  max?: number;
}) {
  const points = data.map((v, i) => ({ i, v }));
  return (
    <ResponsiveContainer width="100%" height={48}>
      <AreaChart data={points} margin={{ top: 4, right: 0, bottom: 0, left: 0 }}>
        <YAxis hide domain={[0, max ?? "dataMax"]} />
        <Area
          type="monotone"
          dataKey="v"
          stroke={color}
          strokeWidth={1.5}
          fill={color}
          fillOpacity={0.15}
          isAnimationActive={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
