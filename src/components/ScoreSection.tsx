import type { IPReport, ScoreValue } from '../types';

function toScore(value: ScoreValue): number | null {
  if (value === null || value === undefined || value === '') return null;
  const num = Number(value);
  if (!Number.isFinite(num)) return null;
  return Math.min(100, Math.max(0, Math.round(num)));
}

function getBarColor(score: number): string {
  if (score <= 20) return 'bg-[#4ade80]';
  if (score <= 50) return 'bg-[#fbbf24]';
  return 'bg-[#f87171]';
}

function getLabel(score: number): string {
  if (score <= 10) return '极低';
  if (score <= 20) return '低';
  if (score <= 50) return '中';
  if (score <= 80) return '高';
  return '极高';
}

function getScoreColor(score: number): string {
  if (score <= 20) return '#4ade80';
  if (score <= 50) return '#fbbf24';
  return '#f87171';
}

function ScoreRow({ label, value }: { label: string; value: ScoreValue }) {
  const num = toScore(value);
  if (num === null) return null;
  return (
    <div className="space-y-1.5">
      <div className="flex justify-between items-center">
        <span className="data-label">{label}</span>
        <span className="text-[13px] font-medium text-[#e5e5e5]">{num}</span>
      </div>
      <div className="h-1 bg-[#2a2a2a] rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-700 ${getBarColor(num)}`}
          style={{ width: `${num}%` }}
        />
      </div>
    </div>
  );
}

export default function ScoreSection({ score }: { score: IPReport['Score'] }) {
  const entries = Object.entries(score);
  const totalEntry = entries.find(([k]) => k.toLowerCase().includes('total'));
  const totalScore = totalEntry ? toScore(totalEntry[1]) : null;
  const ringSize = 120;
  const ringStroke = 10;
  const ringRadius = (ringSize - ringStroke) / 2;
  const ringCircumference = 2 * Math.PI * ringRadius;
  const ringOffset = totalScore === null ? ringCircumference : ringCircumference * (1 - totalScore / 100);

  return (
    <div className="section p-4">
      <div className="section-title">风险评分</div>

      {totalScore !== null && (
        <div className="flex items-center gap-4 mb-4">
          <div className="relative h-20 w-20 shrink-0">
            <svg
              viewBox={`0 0 ${ringSize} ${ringSize}`}
              className="h-full w-full -rotate-90"
              aria-hidden="true"
            >
              <circle
                cx={ringSize / 2}
                cy={ringSize / 2}
                r={ringRadius}
                fill="none"
                stroke="#2a2a2a"
                strokeWidth={ringStroke}
              />
              <circle
                cx={ringSize / 2}
                cy={ringSize / 2}
                r={ringRadius}
                fill="none"
                stroke={getScoreColor(totalScore)}
                strokeWidth={ringStroke}
                strokeLinecap="round"
                strokeDasharray={ringCircumference}
                strokeDashoffset={ringOffset}
                className="transition-[stroke-dashoffset,stroke] duration-700 ease-out"
              />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span
                className="text-2xl font-semibold leading-none"
                style={{ color: getScoreColor(totalScore) }}
              >
                {totalScore}
              </span>
              <span className="mt-0.5 text-[10px] text-[#8a8a8a]">总分</span>
            </div>
          </div>
          <div className="min-w-0">
            <div className="text-[13px] text-[#666]">{getLabel(totalScore)}风险</div>
            <div className="mt-1 text-[12px] text-[#444]">
              评分越高，风险越大
            </div>
          </div>
        </div>
      )}

      <div className="space-y-3">
        {entries
          .filter(([k]) => k !== (totalEntry?.[0] || ''))
          .map(([key, val]) => (
            <ScoreRow key={key} label={key} value={val} />
          ))}
      </div>
    </div>
  );
}
