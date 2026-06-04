import type { IPReport } from '../types';

interface Props {
  head: IPReport['Head'];
  info: IPReport['Info'];
  type: IPReport['Type'];
}

export default function IPOverview({ head, info, type }: Props) {
  const location = [info.City?.Name, info.Region?.Name].filter(Boolean).join(', ');
  const usage = Object.values(type.Usage || {}).join(', ') || '-';

  const rows: [string, string][] = [
    ['IP', head.IP],
    ['ASN', info.ASN],
    ['组织', info.Organization],
    ['位置', location || '-'],
    ['坐标', `${info.Latitude}, ${info.Longitude}`],
    ['时区', info.TimeZone || '-'],
    ['类型', usage],
  ];

  return (
    <div className="section p-3 sm:p-4">
      <div className="space-y-2 sm:space-y-0 sm:grid sm:grid-cols-3 sm:gap-x-6 sm:gap-y-2.5">
        {rows.map(([label, value]) => (
          <div key={label} className="flex items-baseline gap-2 sm:flex-col sm:gap-0.5">
            <span className="data-label whitespace-nowrap shrink-0 w-8 sm:w-auto">{label}</span>
            <span className="data-value">{value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
