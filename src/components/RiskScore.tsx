import type { IPReport } from '../types';

// Risk score: higher = more risky → red. Lower = safer → green.
function getScoreColor(score: number): string {
  if (score <= 20) return 'bg-emerald-500';
  if (score <= 50) return 'bg-amber-500';
  return 'bg-red-500';
}

function getScoreTextColor(score: number): string {
  if (score <= 20) return 'text-emerald-400';
  if (score <= 50) return 'text-amber-400';
  return 'text-red-400';
}

function getScoreLabel(score: number): string {
  if (score <= 10) return '极低风险';
  if (score <= 20) return '低风险';
  if (score <= 50) return '中等风险';
  if (score <= 80) return '高风险';
  return '极高风险';
}

function ScoreBar({ label, value }: { label: string; value: string }) {
  const num = parseInt(value, 10);
  if (isNaN(num)) return null;
  return (
    <div className="space-y-1.5">
      <div className="flex justify-between items-center">
        <span className="text-sm text-slate-400">{label}</span>
        <span className={`text-sm font-bold ${getScoreTextColor(num)}`}>{num}/100</span>
      </div>
      <div className="h-2.5 bg-slate-700/50 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-1000 ease-out ${getScoreColor(num)}`}
          style={{ width: `${num}%` }}
        />
      </div>
    </div>
  );
}

export default function RiskScore({ score }: { score: IPReport['Score'] }) {
  const entries = Object.entries(score);
  const totalEntry = entries.find(([k]) => k.toLowerCase().includes('total') || k.toLowerCase().includes('risk'));
  const totalScore = totalEntry ? parseInt(totalEntry[1], 10) : null;

  return (
    <div className="card animate-fade-in">
      <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4">风险评分</h2>
      {totalScore !== null && (
        <div className="flex items-center justify-center mb-6">
          <div className="relative w-32 h-32">
            <svg className="w-full h-full -rotate-90" viewBox="0 0 120 120">
              <circle cx="60" cy="60" r="52" fill="none" stroke="#334155" strokeWidth="10" />
              <circle
                cx="60" cy="60" r="52" fill="none"
                stroke={totalScore <= 20 ? '#10b981' : totalScore <= 50 ? '#f59e0b' : '#ef4444'}
                strokeWidth="10"
                strokeLinecap="round"
                strokeDasharray={`${(totalScore / 100) * 326.7} 326.7`}
                className="transition-all duration-1500 ease-out"
              />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className={`text-3xl font-bold ${getScoreTextColor(totalScore)}`}>{totalScore}</span>
              <span className="text-xs text-slate-500">{getScoreLabel(totalScore)}</span>
            </div>
          </div>
        </div>
      )}
      <div className="space-y-3">
        {entries.filter(([k]) => k !== (totalEntry?.[0] || '')).map(([key, val]) => (
          <ScoreBar key={key} label={key} value={val} />
        ))}
      </div>
    </div>
  );
}
