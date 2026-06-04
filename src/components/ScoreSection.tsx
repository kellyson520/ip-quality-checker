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

  return (
    <div className="section p-4">
      <div className="section-title">风险评分</div>

      {totalScore !== null && (
        <div className="flex items-center gap-4 mb-4">
          <span className={`text-3xl font-semibold ${
            totalScore <= 20 ? 'text-[#4ade80]' : totalScore <= 50 ? 'text-[#fbbf24]' : 'text-[#f87171]'
          }`}>
            {totalScore}
          </span>
          <span className="text-[13px] text-[#666]">{getLabel(totalScore)}风险</span>
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
