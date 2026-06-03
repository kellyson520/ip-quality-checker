import { Globe, Play, Loader2 } from 'lucide-react';

interface HeaderProps {
  onRun: () => void;
  loading: boolean;
  ip?: string;
}

export default function Header({ onRun, loading, ip }: HeaderProps) {
  return (
    <header className="flex items-center justify-between px-6 py-4 bg-[#1e293b]/80 backdrop-blur-sm border-b border-slate-700/50 sticky top-0 z-50">
      <div className="flex items-center gap-3">
        <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center">
          <Globe className="w-5 h-5 text-white" />
        </div>
        <div>
          <h1 className="text-lg font-bold gradient-text">IP 质量检测</h1>
          {ip && <p className="text-xs text-slate-500">{ip}</p>}
        </div>
      </div>
      <button onClick={onRun} disabled={loading} className="btn-primary flex items-center gap-2 text-sm">
        {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
        {loading ? '检测中...' : '开始检测'}
      </button>
    </header>
  );
}
