import type { IPReport } from '../types';

type MailValue = boolean | null | { Total?: number; Clean?: number; Marked?: number; Blacklisted?: number };

const MAIL_SERVICES = ['Gmail', 'Outlook', 'Yahoo', 'Apple', 'QQ', 'MailRU', 'AOL', 'GMX', 'MailCOM', '163', 'Sohu', 'Sina'];

function StatusDot({ value }: { value: boolean | null }) {
  if (value === true) return <span className="w-1.5 h-1.5 rounded-full bg-[#4ade80]" />;
  if (value === false) return <span className="w-1.5 h-1.5 rounded-full bg-[#f87171]" />;
  return <span className="w-1.5 h-1.5 rounded-full bg-[#444]" />;
}

function StatusText({ value }: { value: boolean | null }) {
  if (value === true) return <span className="text-[13px] text-[#4ade80]">通过</span>;
  if (value === false) return <span className="text-[13px] text-[#f87171]">拒绝</span>;
  return <span className="text-[13px] text-[#444]">-</span>;
}

export default function MailSection({ mail }: { mail: IPReport['Mail'] }) {
  if (!mail || typeof mail !== 'object') return null;

  const dnsbl = mail.DNSBlacklist;
  const hasDnsbl = dnsbl && typeof dnsbl === 'object' && dnsbl.Total !== undefined;

  // Collect service entries (exclude Port25 and DNSBlacklist)
  const serviceEntries = MAIL_SERVICES
    .filter(name => name in mail)
    .map(name => ({ name, val: mail[name] as boolean | null }));

  const hasServices = serviceEntries.length > 0;
  const hasPort25 = 'Port25' in mail;

  if (!hasServices && !hasPort25 && !hasDnsbl) return null;

  return (
    <div className="section p-4">
      <div className="section-title">邮件端口</div>

      {hasPort25 && (
        <div className="flex items-center justify-between py-1.5">
          <div className="flex items-center gap-2">
            <StatusDot value={mail.Port25 as boolean | null} />
            <span className="text-[13px] text-[#ccc]">Port 25</span>
          </div>
          <StatusText value={mail.Port25 as boolean | null} />
        </div>
      )}

      {hasServices && (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-x-4 gap-y-1 mt-2">
          {serviceEntries.map(({ name, val }) => (
            <div key={name} className="flex items-center justify-between py-1">
              <div className="flex items-center gap-1.5">
                <StatusDot value={val} />
                <span className="text-[12px] text-[#999]">{name}</span>
              </div>
            </div>
          ))}
        </div>
      )}

      {hasDnsbl && (
        <div className="flex items-center gap-4 mt-3 pt-3 border-t border-[#2a2a2a]">
          <span className="text-[12px] text-[#666]">DNS 黑名单</span>
          <div className="flex items-center gap-3 text-[12px]">
            <span className="text-[#888]">总计 {dnsbl.Total}</span>
            <span className="text-[#4ade80]">干净 {dnsbl.Clean ?? '-'}</span>
            <span className="text-[#fbbf24]">标记 {dnsbl.Marked ?? '-'}</span>
            <span className="text-[#f87171]">拉黑 {dnsbl.Blacklisted ?? '-'}</span>
          </div>
        </div>
      )}
    </div>
  );
}
