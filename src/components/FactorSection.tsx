import type { IPReport } from '../types';

type BoolVal = boolean | null;

function StatusDot({ value }: { value: BoolVal }) {
  if (value === true) return <span className="w-1.5 h-1.5 rounded-full bg-[#f87171] shrink-0" />;
  if (value === false) return <span className="w-1.5 h-1.5 rounded-full bg-[#4ade80] shrink-0" />;
  return <span className="w-1.5 h-1.5 rounded-full bg-[#444] shrink-0" />;
}

function FactorRow({ label, sources }: { label: string; sources: Record<string, BoolVal> }) {
  const entries = Object.entries(sources);
  if (entries.length === 0) return null;
  return (
    <div className="py-1.5">
      <div className="flex items-center justify-between">
        <span className="data-label">{label}</span>
        <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 justify-end">
          {entries.map(([key, val]) => (
            <div key={key} className="flex items-center gap-1">
              <StatusDot value={val} />
              <span className="text-[10px] sm:text-[11px] text-[#555]">{key}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function toRecord(v: unknown): Record<string, BoolVal> {
  if (v && typeof v === 'object' && !Array.isArray(v)) return v as Record<string, BoolVal>;
  return {};
}

export default function FactorSection({ factor }: { factor: IPReport['Factor'] }) {
  const sections: [string, Record<string, BoolVal>][] = [
    ['代理', toRecord(factor.Proxy)],
    ['Tor', toRecord(factor.Tor)],
    ['VPN', toRecord(factor.VPN)],
    ['服务器', toRecord(factor.Server)],
    ['滥用', toRecord(factor.Abuser)],
    ['机器人', toRecord(factor.Robot)],
  ];

  const hasData = sections.some(([, data]) => Object.keys(data).length > 0);

  return (
    <div className="section p-3 sm:p-4">
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
