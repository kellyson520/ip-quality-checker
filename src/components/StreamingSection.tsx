import type { IPReport } from '../types';

const SERVICES: Record<string, string> = {
  TikTok: 'TikTok',
  DisneyPlus: 'Disney+',
  Netflix: 'Netflix',
  YouTube: 'YouTube',
  AmazonPrime: 'Prime Video',
  Reddit: 'Reddit',
  ChatGPT: 'ChatGPT',
};

function Badge({ result }: { result: string }) {
  const r = result.toLowerCase();
  if (r === 'y' || r.includes('yes') || r.includes('ok') || r.includes('unlock'))
    return <span className="badge badge-green">解锁</span>;
  if (r === 'n' || r.includes('no') || r.includes('block') || r.includes('denied'))
    return <span className="badge badge-red">锁定</span>;
  return <span className="badge badge-gray">{result}</span>;
}

export default function StreamingSection({ media }: { media: IPReport['Media'] }) {
  const entries = Object.entries(media || {});
  if (entries.length === 0) return null;

  return (
    <div className="section p-4">
      <div className="section-title">流媒体</div>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">
        {entries.map(([key, val]) => (
          <div key={key} className="flex items-center justify-between py-1.5 px-3 rounded bg-[#1f1f1f]">
            <span className="text-[13px] text-[#ccc]">{SERVICES[key] || key}</span>
            <Badge result={val.Result} />
          </div>
        ))}
      </div>
    </div>
  );
}
