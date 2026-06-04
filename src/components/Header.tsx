import { Loader2 } from 'lucide-react';

interface HeaderProps {
  onRun: () => void;
  loading: boolean;
  ip?: string;
}

export default function Header({ onRun, loading, ip }: HeaderProps) {
  return (
    <header className="flex items-center justify-between px-5 py-3 border-b border-[#2a2a2a]">
      <div className="flex items-center gap-3">
        <span className="text-sm font-semibold text-white tracking-tight">IPQC</span>
        {ip && <span className="text-[13px] text-[#666] font-mono">{ip}</span>}
      </div>
      <button onClick={onRun} disabled={loading} className="btn-run flex items-center gap-2">
        {loading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : null}
        {loading ? '检测中' : '检测'}
      </button>
    </header>
  );
}
