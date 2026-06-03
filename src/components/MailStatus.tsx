import { Mail, CheckCircle2, XCircle, HelpCircle } from 'lucide-react';
import type { IPReport } from '../types';

function StatusIcon({ status }: { status: string }) {
  const s = status.toLowerCase();
  if (s.includes('ok') || s.includes('success') || s.includes('connect'))
    return <CheckCircle2 className="w-4 h-4 text-emerald-400" />;
  if (s.includes('fail') || s.includes('error') || s.includes('timeout') || s.includes('refuse'))
    return <XCircle className="w-4 h-4 text-red-400" />;
  return <HelpCircle className="w-4 h-4 text-amber-400" />;
}

export default function MailStatus({ mail }: { mail: IPReport['Mail'] }) {
  const entries = Object.entries(mail || {});
  if (entries.length === 0) return null;

  return (
    <div className="card animate-fade-in">
      <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4 flex items-center gap-2">
        <Mail className="w-4 h-4" />
        邮件服务连通性
      </h2>
      <div className="space-y-2">
        {entries.map(([key, val]) => (
          <div
            key={key}
            className="flex items-center justify-between py-2.5 px-3 rounded-lg bg-slate-800/30 hover:bg-slate-800/50 transition-colors"
          >
            <div className="flex items-center gap-2.5">
              <StatusIcon status={val.Status} />
              <span className="text-sm text-slate-300">{key}</span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-xs text-slate-500">端口 {val.Port}</span>
              <span className="text-sm text-slate-400">{val.Status}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
