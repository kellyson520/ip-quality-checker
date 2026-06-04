import type { IPReport } from '../types';
import { cleanScalar } from '../report';

interface Props {
  head: IPReport['Head'];
  info: IPReport['Info'];
  type: IPReport['Type'];
}

export default function IPOverview({ head, info, type }: Props) {
  const location = [info.City?.Name, info.Region?.Name].map(cleanScalar).filter(Boolean).join(', ');
  const usageValues = Object.values(type.Usage || {}).map(cleanScalar).filter(Boolean);
  const infoType = cleanScalar(info.Type);
  const usage = usageValues.join(', ') || infoType || '-';
  const infoTypeBadgeClass = infoType?.includes('本土')
    ? 'badge badge-green'
    : infoType?.includes('海外')
      ? 'badge badge-amber'
      : 'badge badge-gray';
  const latitude = cleanScalar(info.Latitude);
  const longitude = cleanScalar(info.Longitude);
  const coordinates = latitude && longitude ? `${latitude}, ${longitude}` : '-';

  const rows: [string, string][] = [
    ['IP', head.IP],
    ['ASN', cleanScalar(info.ASN) || '-'],
    ['组织', cleanScalar(info.Organization) || '-'],
    ['位置', location || '-'],
    ['坐标', coordinates],
    ['时区', cleanScalar(info.TimeZone) || '-'],
    ['类型', usage],
  ];

  return (
    <div className="section p-3 sm:p-4">
      <div className="grid grid-cols-1 gap-y-2 sm:grid-cols-2 sm:gap-x-6 sm:gap-y-2.5 md:grid-cols-3">
        {rows.map(([label, value]) => (
          <div key={label} className="flex min-w-0 items-baseline gap-2 sm:flex-col sm:gap-0.5">
            <span className="data-label whitespace-nowrap shrink-0 w-8 sm:w-auto">{label}</span>
            <span className="data-value min-w-0 break-words">{value}</span>
          </div>
        ))}
        {infoType && (
          <div className="flex min-w-0 items-baseline gap-2 sm:flex-col sm:gap-0.5">
            <span className="data-label whitespace-nowrap shrink-0 w-8 sm:w-auto">地址类型</span>
            <span className={`data-value ${infoTypeBadgeClass}`}>{infoType}</span>
          </div>
        )}
      </div>
    </div>
  );
}
