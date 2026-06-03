import { MapPin, Building2, Globe2, Wifi, Clock, Hash } from 'lucide-react';
import type { IPReport } from '../types';

interface Props {
  head: IPReport['Head'];
  info: IPReport['Info'];
  type: IPReport['Type'];
}

function InfoRow({ icon: Icon, label, value }: { icon: any; label: string; value: string }) {
  return (
    <div className="flex items-center gap-3 py-2">
      <Icon className="w-4 h-4 text-blue-400 shrink-0" />
      <span className="text-slate-500 text-sm w-20 shrink-0">{label}</span>
      <span className="text-slate-200 text-sm truncate">{value}</span>
    </div>
  );
}

export default function IPInfoCard({ head, info, type }: Props) {
  const location = [info.City?.Name, info.Region?.Name, info.Continent?.Name].filter(Boolean).join(', ');
  const usage = Object.values(type.Usage || {}).join(', ') || '未知';
  const company = Object.values(type.Company || {}).join(', ') || '未知';

  return (
    <div className="card animate-fade-in">
      <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4">基本信息</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8">
        <InfoRow icon={Hash} label="IP 地址" value={head.IP} />
        <InfoRow icon={Building2} label="ASN" value={info.ASN} />
        <InfoRow icon={Globe2} label="组织" value={info.Organization} />
        <InfoRow icon={MapPin} label="位置" value={location || '未知'} />
        <InfoRow icon={Wifi} label="经纬度" value={`${info.Latitude}, ${info.Longitude}`} />
        <InfoRow icon={Clock} label="检测时间" value={head.Time} />
        <InfoRow icon={Building2} label="使用类型" value={usage} />
        <InfoRow icon={Building2} label="公司" value={company} />
      </div>
    </div>
  );
}
