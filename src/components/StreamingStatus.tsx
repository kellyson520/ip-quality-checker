import { Tv } from 'lucide-react';
import type { IPReport } from '../types';

const SERVICE_CONFIG: Record<string, { emoji: string; name: string }> = {
  TikTok: { emoji: '🎵', name: 'TikTok' },
  DisneyPlus: { emoji: '🏰', name: 'Disney+' },
  Netflix: { emoji: '🎬', name: 'Netflix' },
  YouTube: { emoji: '▶️', name: 'YouTube' },
  AmazonPrime: { emoji: '📦', name: 'Prime Video' },
  Reddit: { emoji: '🤖', name: 'Reddit' },
  ChatGPT: { emoji: '💬', name: 'ChatGPT' },
};

function resultBadge(result: string) {
  const lower = result.toLowerCase();
  if (lower === 'y' || lower.includes('yes') || lower.includes('ok') || lower.includes('unlock') || lower.includes('原生'))
    return <span className="badge-good text-xs px-2 py-0.5 rounded-full">解锁</span>;
  if (lower === 'n' || lower.includes('no') || lower.includes('block') || lower.includes('denied'))
    return <span className="badge-bad text-xs px-2 py-0.5 rounded-full">锁定</span>;
  return <span className="badge-warn text-xs px-2 py-0.5 rounded-full">{result}</span>;
}

export default function StreamingStatus({ media }: { media: IPReport['Media'] }) {
  const entries = Object.entries(media || {});
  if (entries.length === 0) return null;

  return (
    <div className="card animate-fade-in">
      <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4 flex items-center gap-2">
        <Tv className="w-4 h-4" />
        流媒体解锁状态
      </h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {entries.map(([key, val]) => {
          const cfg = SERVICE_CONFIG[key] || { emoji: '📺', name: key };
          return (
            <div
              key={key}
              className="flex items-center justify-between py-2.5 px-3 rounded-lg bg-slate-800/30 hover:bg-slate-800/50 transition-colors"
            >
              <div className="flex items-center gap-2.5">
                <span className="text-lg">{cfg.emoji}</span>
                <span className="text-sm text-slate-300">{cfg.name}</span>
              </div>
              {resultBadge(val.Result)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
