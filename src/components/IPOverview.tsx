import type { IPReport } from '../types';

interface Props {
  head: IPReport['Head'];
  info: IPReport['Info'];
  type: IPReport['Type'];
}

export default function IPOverview({ head, info, type }: Props) {
  const location = [info.City?.Name, info.Region?.Name].filter(Boolean).join(', ');
  const usageValues = Object.values(type.Usage || {})
    .filter((value) => value !== null && value !== undefined && value !== '' && value !== 'null')
    .map(String);
  const usage = usageValues.join(', ') || info.Type || '-';
  const infoType = info.Type || '-';
  const infoTypeBadgeClass = info.Type?.includes('本土')
    ? 'badge badge-green'
    : info.Type?.includes('海外')
      ? 'badge badge-amber'
      : 'badge badge-gray';
  const coordinates =
    info.Latitude && info.Longitude && info.Latitude !== 'null' && info.Longitude !== 'null'
      ? `${info.Latitude}, ${info.Longitude}`
      : '-';

  const rows: [string, string][] = [
    ['IP', head.IP],
    ['ASN', info.ASN],
    ['组织', info.Organization],
    ['位置', location || '-'],
    ['坐标', coordinates],
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
        <div className="flex items-baseline gap-2 sm:flex-col sm:gap-0.5">
          <span className="data-label whitespace-nowrap shrink-0 w-8 sm:w-auto">地址类型</span>
          <span className={`data-value ${infoTypeBadgeClass}`}>{infoType}</span>
        </div>
      </div>
    </div>
  );
}
