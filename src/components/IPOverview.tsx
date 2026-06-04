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
    ['类型', usage],
  ];

  return (
    <div className="section p-4">
      <div className="grid grid-cols-2 md:grid-cols-3 gap-x-8 gap-y-2.5">
        {rows.map(([label, value]) => (
          <div key={label} className="flex flex-col gap-0.5">
            <span className="data-label">{label}</span>
            <span className="data-value truncate">{value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
