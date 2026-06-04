import type { IPReport, RiskFlag } from '../types';
import { normalizePassFlag } from '../report';

const MAIL_SERVICES = ['Gmail', 'Outlook', 'Yahoo', 'Apple', 'QQ', 'MailRU', 'AOL', 'GMX', 'MailCOM', '163', 'Sohu', 'Sina'];

function StatusDot({ value }: { value: RiskFlag }) {
  const normalized = normalizePassFlag(value);
  if (normalized === true) return <span className="w-1.5 h-1.5 rounded-full bg-[#4ade80] shrink-0" />;
  if (normalized === false) return <span className="w-1.5 h-1.5 rounded-full bg-[#f87171] shrink-0" />;
  return <span className="w-1.5 h-1.5 rounded-full bg-[#444] shrink-0" />;
}

function StatusText({ value }: { value: RiskFlag }) {
  const normalized = normalizePassFlag(value);
  if (normalized === true) return <span className="text-[13px] text-[#4ade80]">通过</span>;
  if (normalized === false) return <span className="text-[13px] text-[#f87171]">拒绝</span>;
  return <span className="text-[13px] text-[#444]">-</span>;
}

export default function MailSection({ mail }: { mail: IPReport['Mail'] }) {
  if (!mail || typeof mail !== 'object') return null;

  const dnsbl = mail.DNSBlacklist;
  const hasDnsbl = dnsbl && typeof dnsbl === 'object' && dnsbl.Total !== undefined;

  const serviceEntries = MAIL_SERVICES
    .filter(name => name in mail)
    .map(name => ({ name, val: mail[name] as RiskFlag }));

  const hasServices = serviceEntries.length > 0;
  const hasPort25 = 'Port25' in mail;

  if (!hasServices && !hasPort25 && !hasDnsbl) return null;

  return (
    <div className="section p-3 sm:p-4">
      <div className="section-title">邮件端口</div>

      {hasPort25 && (
        <div className="flex items-center justify-between py-1.5">
          <div className="flex items-center gap-2">
            <StatusDot value={mail.Port25} />
            <span className="text-[13px] text-[#ccc]">Port 25</span>
          </div>
          <StatusText value={mail.Port25} />
        </div>
      )}

      {hasServices && (
        <div className="mt-2 grid grid-cols-2 gap-x-3 gap-y-1.5 sm:grid-cols-4 md:grid-cols-6">
          {serviceEntries.map(({ name, val }) => (
            <div key={name} className="flex items-center gap-1.5">
              <StatusDot value={val} />
              <span className="text-[11px] sm:text-[12px] text-[#999] truncate">{name}</span>
            </div>
          ))}
        </div>
      )}

      {hasDnsbl && (
        <div className="mt-3 pt-3 border-t border-[#2a2a2a]">
          <span className="text-[11px] text-[#666] block mb-1.5">DNS 黑名单</span>
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[12px]">
            <span className="text-[#888]">总计 {dnsbl.Total ?? '-'}</span>
            <span className="text-[#4ade80]">干净 {dnsbl.Clean ?? '-'}</span>
            <span className="text-[#fbbf24]">标记 {dnsbl.Marked ?? '-'}</span>
            <span className="text-[#f87171]">拉黑 {dnsbl.Blacklisted ?? '-'}</span>
          </div>
        </div>
      )}
    </div>
  );
}
