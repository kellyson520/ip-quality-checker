import type { IPReport } from '../types';

interface Props {
  head: IPReport['Head'];
  info: IPReport['Info'];
  type: IPReport['Type'];
}

export default function IPOverview({ head, info, type }: Props) {
  const cleanValue = (value: unknown): string | null => {
    if (value === null || value === undefined) return null;
    const text = String(value).trim();
    return text && text.toLowerCase() !== 'null' ? text : null;
  };

  const location = [info.City?.Name, info.Region?.Name].map(cleanValue).filter(Boolean).join(', ');
  const usageValues = Object.values(type.Usage || {}).map(cleanValue).filter(Boolean);
  const infoType = cleanValue(info.Type);
  const usage = usageValues.join(', ') || infoType || '-';
  const infoTypeBadgeClass = infoType?.includes('本土')
    ? 'badge badge-green'
    : infoType?.includes('海外')
      ? 'badge badge-amber'
      : 'badge badge-gray';
  const latitude = cleanValue(info.Latitude);
  const longitude = cleanValue(info.Longitude);
  const coordinates = latitude && longitude ? `${latitude}, ${longitude}` : '-';

  const rows: [string, string][] = [
    ['IP', head.IP],
    ['ASN', cleanValue(info.ASN) || '-'],
    ['组织', cleanValue(info.Organization) || '-'],
    ['位置', location || '-'],
    ['坐标', coordinates],
    ['时区', cleanValue(info.TimeZone) || '-'],
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
