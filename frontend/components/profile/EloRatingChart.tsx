import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ReferenceLine,
  type TooltipContentProps,
} from 'recharts';

export interface EloDataPoint {
  date: string
  elo: number
  opponent: string
  change: number
}

export const MOCK_ELO_DATA: EloDataPoint[] = [
  { date: '2026-02-25', elo: 1150, opponent: 'GABC...AA01', change: +18 },
  { date: '2026-02-26', elo: 1132, opponent: 'GDEF...BB02', change: -18 },
  { date: '2026-02-27', elo: 1148, opponent: 'GHIJ...CC03', change: +16 },
  { date: '2026-02-28', elo: 1165, opponent: 'GKLM...DD04', change: +17 },
  { date: '2026-03-01', elo: 1149, opponent: 'GNOP...EE05', change: -16 },
  { date: '2026-03-02', elo: 1166, opponent: 'GQRS...FF06', change: +17 },
  { date: '2026-03-03', elo: 1183, opponent: 'GTUV...GG07', change: +17 },
  { date: '2026-03-04', elo: 1168, opponent: 'GWXY...HH08', change: -15 },
  { date: '2026-03-05', elo: 1185, opponent: 'GZAB...II09', change: +17 },
  { date: '2026-03-06', elo: 1200, opponent: 'GCDE...JJ10', change: +15 },
  { date: '2026-03-07', elo: 1185, opponent: 'GFGH...KK11', change: -15 },
  { date: '2026-03-08', elo: 1201, opponent: 'GIJK...LL12', change: +16 },
  { date: '2026-03-09', elo: 1218, opponent: 'GLMN...MM13', change: +17 },
  { date: '2026-03-10', elo: 1203, opponent: 'GOPQ...NN14', change: -15 },
  { date: '2026-03-11', elo: 1220, opponent: 'GRST...OO15', change: +17 },
  { date: '2026-03-12', elo: 1204, opponent: 'GUVW...PP16', change: -16 },
  { date: '2026-03-13', elo: 1221, opponent: 'GXYZ...QQ17', change: +17 },
  { date: '2026-03-14', elo: 1238, opponent: 'GABC...RR18', change: +17 },
  { date: '2026-03-15', elo: 1222, opponent: 'GDEF...SS19', change: -16 },
  { date: '2026-03-16', elo: 1239, opponent: 'GHIJ...TT20', change: +17 },
  { date: '2026-03-17', elo: 1255, opponent: 'GKLM...UU21', change: +16 },
  { date: '2026-03-18', elo: 1240, opponent: 'GNOP...VV22', change: -15 },
  { date: '2026-03-19', elo: 1257, opponent: 'GQRS...WW23', change: +17 },
  { date: '2026-03-20', elo: 1242, opponent: 'GTUV...XX24', change: -15 },
  { date: '2026-03-21', elo: 1259, opponent: 'GWXY...YY25', change: +17 },
  { date: '2026-03-22', elo: 1244, opponent: 'GZAB...ZZ26', change: -15 },
  { date: '2026-03-23', elo: 1261, opponent: 'GCDE...AA27', change: +17 },
  { date: '2026-03-24', elo: 1278, opponent: 'GFGH...BB28', change: +17 },
  { date: '2026-03-25', elo: 1263, opponent: 'GIJK...CC29', change: -15 },
  { date: '2026-03-26', elo: 1280, opponent: 'GLMN...DD30', change: +17 },
]

// ── Custom tooltip ────────────────────────────────────────────────────────────

function CustomTooltip({ active, payload }: TooltipContentProps) {
  if (!active || !payload?.length) return null;

  const point = payload[0].payload as EloDataPoint;
  const isGain = point.change >= 0;

  return (
    <div className="rounded-lg border border-gray-200 bg-white px-4 py-3 shadow-lg text-sm">
      <p className="font-semibold text-gray-900 mb-1">
        {new Date(point.date).toLocaleDateString('en-US', {
          month: 'short',
          day: 'numeric',
          year: 'numeric',
        })}
      </p>
      <p className="text-gray-600">
        Rating: <span className="font-semibold text-gray-900">{point.elo}</span>
      </p>
      <p className={`font-semibold ${isGain ? 'text-emerald-600' : 'text-red-500'}`}>
        {isGain ? '+' : ''}{point.change} pts
      </p>
      <p className="text-gray-400 text-xs mt-1 truncate max-w-[180px]">
        vs {point.opponent}
      </p>
    </div>
  );
}

// ── Chart tick formatter ──────────────────────────────────────────────────────

function formatDateTick(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
}

// ── Main component ────────────────────────────────────────────────────────────

interface EloRatingChartProps {
  data?: EloDataPoint[];
  title?: string;
}

export function EloRatingChart({
  data = MOCK_ELO_DATA,
  title = 'Elo Rating — Last 30 Games',
}: EloRatingChartProps) {
  const elos = data.map((d) => d.elo);
  const minElo = Math.min(...elos);
  const maxElo = Math.max(...elos);
  const PADDING = 20;
  const yDomain: [number, number] = [minElo - PADDING, maxElo + PADDING];
  const startingElo = data[0]?.elo ?? 1200;

  const xTicks = data
    .filter((_, i) => i % 5 === 0 || i === data.length - 1)
    .map((d) => d.date);

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-base font-semibold text-gray-900">{title}</h2>
          <p className="text-xs text-gray-400 mt-0.5">
            Current rating:{' '}
            <span className="font-semibold text-gray-700">
              {data[data.length - 1]?.elo ?? '—'}
            </span>
          </p>
        </div>

        {/* Net change badge */}
        {data.length > 1 && (() => {
          const net = (data[data.length - 1]?.elo ?? 0) - (data[0]?.elo ?? 0);
          const isPositive = net >= 0;
          return (
            <span
              className={`inline-flex items-center rounded-full px-3 py-1 text-sm font-semibold ${
                isPositive
                  ? 'bg-emerald-50 text-emerald-700'
                  : 'bg-red-50 text-red-600'
              }`}
            >
              {isPositive ? '+' : ''}{net} overall
            </span>
          );
        })()}
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height={280}>
        <LineChart
          data={data}
          margin={{ top: 8, right: 16, bottom: 4, left: 0 }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />

          <XAxis
            dataKey="date"
            ticks={xTicks}
            tickFormatter={formatDateTick}
            tick={{ fontSize: 11, fill: '#9ca3af' }}
            axisLine={false}
            tickLine={false}
          />

          <YAxis
            domain={yDomain}
            tick={{ fontSize: 11, fill: '#9ca3af' }}
            axisLine={false}
            tickLine={false}
            width={40}
          />

          <Tooltip content={CustomTooltip} />

          {/* Reference line at the player's starting Elo */}
          <ReferenceLine
            y={startingElo}
            stroke="#e5e7eb"
            strokeDasharray="4 4"
            label={{
              value: `Start ${startingElo}`,
              position: 'insideTopRight',
              fontSize: 10,
              fill: '#d1d5db',
            }}
          />

          <Line
            type="monotone"
            dataKey="elo"
            stroke="#6366f1"
            strokeWidth={2}
            dot={{ r: 3, fill: '#6366f1', strokeWidth: 0 }}
            activeDot={{ r: 6, fill: '#6366f1', stroke: '#fff', strokeWidth: 2 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
