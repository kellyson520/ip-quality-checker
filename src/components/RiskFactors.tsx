import { Shield, ShieldAlert, ShieldCheck, ShieldX, Info } from 'lucide-react';
import type { IPReport } from '../types';

interface Props {
  factor: IPReport['Factor'];
}

type BoolVal = boolean | null;

function StatusIcon({ value }: { value: BoolVal }) {
  if (value === true) return <ShieldX className="w-4 h-4 text-red-400" />;
  if (value === false) return <ShieldCheck className="w-4 h-4 text-emerald-400" />;
  return <Info className="w-4 h-4 text-slate-500" />;
}

function StatusBadge({ value }: { value: BoolVal }) {
  if (value === true) return <span className="badge-bad text-xs px-2 py-0.5 rounded-full">是</span>;
  if (value === false) return <span className="badge-good text-xs px-2 py-0.5 rounded-full">否</span>;
  return <span className="badge-neutral text-xs px-2 py-0.5 rounded-full">未知</span>;
}

function FactorSection({
  title,
  icon: Icon,
  data,
}: {
  title: string;
  icon: typeof Shield;
  data: Record<string, BoolVal>;
}) {
  const entries = Object.entries(data);
  if (entries.length === 0) return null;

  return (
    <div className="space-y-2">
      <h3 className="text-sm font-medium text-slate-300 flex items-center gap-2">
        <Icon className="w-4 h-4 text-blue-400" />
        {title}
      </h3>
      <div className="space-y-1.5">
        {entries.map(([key, val]) => (
          <div key={key} className="flex items-center justify-between py-1 px-2 rounded-lg bg-slate-800/30">
            <div className="flex items-center gap-2">
              <StatusIcon value={val} />
              <span className="text-sm text-slate-400">{key}</span>
            </div>
            <StatusBadge value={val} />
          </div>
        ))}
      </div>
    </div>
  );
}

export default function RiskFactors({ factor }: Props) {
  const proxyEntries = Object.entries(factor.Proxy || {});
  const torEntries = Object.entries(factor.Tor || {});
  const vpnEntries = Object.entries(factor.VPN || {});
  const abuserEntries = Object.entries(factor.Abuser || {});

  const hasData = proxyEntries.length + torEntries.length + vpnEntries.length + abuserEntries.length > 0;

  return (
    <div className="card animate-fade-in">
      <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4 flex items-center gap-2">
        <ShieldAlert className="w-4 h-4" />
        风险因素
      </h2>
      {hasData ? (
        <div className="space-y-5">
          <FactorSection title="代理检测" icon={Shield} data={Object.fromEntries(proxyEntries)} />
          <FactorSection title="Tor 出口" icon={Shield} data={Object.fromEntries(torEntries)} />
          <FactorSection title="VPN 检测" icon={Shield} data={Object.fromEntries(vpnEntries)} />
          <FactorSection title="滥用行为" icon={ShieldAlert} data={Object.fromEntries(abuserEntries)} />
        </div>
      ) : (
        <p className="text-slate-500 text-sm">暂无风险因素数据</p>
      )}
    </div>
  );
}
