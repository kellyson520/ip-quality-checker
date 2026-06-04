import type { IPReport, Scalar } from '../types';
import { cleanScalar } from '../report';

const SERVICES: Record<string, string> = {
  TikTok: 'TikTok',
  Bilibili: 'Bilibili',
  DisneyPlus: 'Disney+',
  Netflix: 'Netflix',
  Youtube: 'YouTube',
  AmazonPrimeVideo: 'Prime Video',
  Reddit: 'Reddit',
  ChatGPT: 'ChatGPT',
};

function displayValue(value: Scalar): string {
  return cleanScalar(value) ?? '';
}

function Badge({ status }: { status: Scalar }) {
  const text = displayValue(status);
  const s = text.toLowerCase();
  if (!s || s === 'null' || s === 'nodata')
    return <span className="badge badge-gray">-</span>;
  if (s.includes('解锁') || s.includes('yes') || s.includes('ok') || s.includes('unlock'))
    return <span className="badge badge-green">{text}</span>;
  if (s.includes('block') || s.includes('锁定') || s.includes('no') || s.includes('denied') || s.includes('china'))
    return <span className="badge badge-red">{text}</span>;
  if (s.includes('idc') || s.includes('pending') || s.includes('noprem') || s.includes('nf.only') || s.includes('webonly') || s.includes('apponly'))
    return <span className="badge badge-amber">{text}</span>;
  return <span className="badge badge-gray">{text}</span>;
}

function RegionBadge({ region }: { region?: Scalar }) {
  const text = displayValue(region);
  if (!text || text === 'null') return null;
  return (
    <span
      className="ml-1 inline-flex items-center rounded-full border border-[#2f2f2f] bg-[#1a1a1a] px-1.5 py-0.5 text-[10px] font-medium leading-none text-[#9a9a9a] max-w-[96px] truncate align-middle"
      title={text}
    >
      {text}
    </span>
  );
}

export default function StreamingSection({ media }: { media: IPReport['Media'] }) {
  const entries = Object.entries(media || {});
  if (entries.length === 0) return null;

  return (
    <div className="section p-3 sm:p-4">
      <div className="section-title">流媒体</div>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {entries.map(([key, val]) => (
          <div
            key={key}
            className="flex min-w-0 items-center justify-between gap-3 rounded bg-[#1f1f1f] px-3 py-2"
          >
            <div className="flex min-w-0 items-center">
              <span className="truncate text-[13px] text-[#ccc]">{SERVICES[key] || key}</span>
              <RegionBadge region={val.Region} />
            </div>
            <Badge status={val.Status ?? val.Result} />
          </div>
        ))}
      </div>
    </div>
  );
}
