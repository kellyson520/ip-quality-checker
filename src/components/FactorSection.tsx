import type { IPReport } from '../types';

type BoolVal = boolean | null;

function StatusDot({ value }: { value: BoolVal }) {
  if (value === true) return <span className="w-1.5 h-1.5 rounded-full bg-[#f87171]" />;
  if (value === false) return <span className="w-1.5 h-1.5 rounded-full bg-[#4ade80]" />;
  return <span className="w-1.5 h-1.5 rounded-full bg-[#444]" />;
}

function FactorRow({ label, sources }: { label: string; sources: Record<string, BoolVal> }) {
  const entries = Object.entries(sources);
  if (entries.length === 0) return null;
  return (
    <div className="flex items-center justify-between py-1.5">
      <span className="data-label">{label}</span>
      <div className="flex items-center gap-3">
        {entries.map(([key, val]) => (
          <div key={key} className="flex items-center gap-1.5">
            <StatusDot value={val} />
            <span className="text-[11px] text-[#555]">{key}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export default function FactorSection({ factor }: { factor: IPReport['Factor'] }) {
  const sections: [string, Record<string, BoolVal>][] = [
    ['代理', Object.fromEntries(Object.entries(factor.Proxy || {}))],
    ['Tor', Object.fromEntries(Object.entries(factor.Tor || {}))],
    ['VPN', Object.fromEntries(Object.entries(factor.VPN || {}))],
    ['滥用', Object.fromEntries(Object.entries(factor.Abuser || {}))],
  ];

  const hasData = sections.some(([, data]) => Object.keys(data).length > 0);

  return (
    <div className="section p-4">
      <div className="section-title">风险因素</div>
      {hasData ? (
        <div className="divide-y divide-[#2a2a2a]">
          {sections.map(([title, data]) => (
            <FactorRow key={title} label={title} sources={data} />
          ))}
        </div>
      ) : (
        <p className="text-[#444] text-[13px]">暂无数据</p>
      )}
    </div>
  );
}
