import { Loader2 } from 'lucide-react';

interface HeaderProps {
  onRun: () => void;
  loading: boolean;
  ip?: string;
}

export default function Header({ onRun, loading, ip }: HeaderProps) {
  return (
    <header className="app-header flex items-center justify-between px-3 sm:px-5 border-b border-[#2a2a2a]">
      <div className="flex items-center gap-2 sm:gap-3 min-w-0">
        <span className="text-sm font-semibold text-white tracking-tight shrink-0">IPQC</span>
        {ip && <span className="text-[12px] sm:text-[13px] text-[#666] font-mono truncate">{ip}</span>}
      </div>
      <button onClick={onRun} disabled={loading} className="btn-run flex items-center gap-1.5 shrink-0 ml-2">
        {loading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : null}
        <span className="text-[13px] sm:text-sm">{loading ? '检测中' : '检测'}</span>
      </button>
    </header>
  );
}
