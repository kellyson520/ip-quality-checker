import type { IPReport } from '../types';

const SERVICES: Record<string, string> = {
  TikTok: 'TikTok',
  DisneyPlus: 'Disney+',
  Netflix: 'Netflix',
  Youtube: 'YouTube',
  AmazonPrimeVideo: 'Prime Video',
  Reddit: 'Reddit',
  ChatGPT: 'ChatGPT',
};

function Badge({ status }: { status: string }) {
  const s = status.trim().toLowerCase();
  if (!s || s === 'null' || s === 'nodata')
    return <span className="badge badge-gray">-</span>;
  if (s.includes('解锁') || s.includes('yes') || s.includes('ok') || s.includes('unlock'))
    return <span className="badge badge-green">{status.trim()}</span>;
  if (s.includes('block') || s.includes('锁定') || s.includes('no') || s.includes('denied') || s.includes('china'))
    return <span className="badge badge-red">{status.trim()}</span>;
  if (s.includes('idc') || s.includes('pending') || s.includes('noprem') || s.includes('nf.only') || s.includes('webonly') || s.includes('apponly'))
    return <span className="badge badge-amber">{status.trim()}</span>;
  return <span className="badge badge-gray">{status.trim()}</span>;
}

export default function StreamingSection({ media }: { media: IPReport['Media'] }) {
  const entries = Object.entries(media || {});
  if (entries.length === 0) return null;

  return (
    <div className="section p-3 sm:p-4">
      <div className="section-title">流媒体</div>
      {/* Mobile: vertical list. Desktop: 2-4 col grid */}
      <div className="space-y-1 sm:space-y-0 sm:grid sm:grid-cols-2 md:grid-cols-4 sm:gap-2">
        {entries.map(([key, val]) => (
          <div key={key} className="flex items-center justify-between py-1.5 sm:py-1.5 sm:px-3 sm:rounded sm:bg-[#1f1f1f]">
            <span className="text-[13px] text-[#ccc]">{SERVICES[key] || key}</span>
            <Badge status={val.Status} />
          </div>
        ))}
      </div>
    </div>
  );
}
