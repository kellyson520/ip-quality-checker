import type { IPReport } from '../types';

function StatusDot({ status }: { status: string }) {
  const s = status.toLowerCase();
  if (s.includes('ok') || s.includes('success') || s.includes('connect'))
    return <span className="w-1.5 h-1.5 rounded-full bg-[#4ade80]" />;
  if (s.includes('fail') || s.includes('error') || s.includes('timeout') || s.includes('refuse'))
    return <span className="w-1.5 h-1.5 rounded-full bg-[#f87171]" />;
  return <span className="w-1.5 h-1.5 rounded-full bg-[#444]" />;
}

export default function MailSection({ mail }: { mail: IPReport['Mail'] }) {
  const entries = Object.entries(mail || {});
  if (entries.length === 0) return null;

  return (
    <div className="section p-4">
      <div className="section-title">邮件端口</div>
      <div className="space-y-0 divide-y divide-[#2a2a2a]">
        {entries.map(([key, val]) => (
          <div key={key} className="flex items-center justify-between py-1.5">
            <div className="flex items-center gap-2">
              <StatusDot status={val.Status} />
              <span className="text-[13px] text-[#ccc]">{key}</span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-[11px] text-[#555]">:{val.Port}</span>
              <span className="text-[13px] text-[#888]">{val.Status}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
