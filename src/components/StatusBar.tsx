import { Activity, Wifi, Clock } from 'lucide-react';

interface Props {
  status: 'idle' | 'loading' | 'done' | 'error';
  version?: string;
  time?: string;
}

export default function StatusBar({ status, version, time }: Props) {
  const statusMap = {
    idle: { text: '就绪', color: 'text-slate-500', icon: Activity },
    loading: { text: '检测中', color: 'text-blue-400', icon: Wifi },
    done: { text: '完成', color: 'text-emerald-400', icon: Activity },
    error: { text: '错误', color: 'text-red-400', icon: Activity },
  };
  const s = statusMap[status];
  const Icon = s.icon;

  return (
    <footer className="flex items-center justify-between px-6 py-2.5 bg-[#1e293b]/60 border-t border-slate-700/50 text-xs">
      <div className="flex items-center gap-2">
        <span className={`flex items-center gap-1 ${s.color}`}>
          <Icon className="w-3 h-3" />
          {s.text}
        </span>
        {status === 'loading' && (
          <span className="flex gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-bounce [animation-delay:0ms]" />
            <span className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-bounce [animation-delay:150ms]" />
            <span className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-bounce [animation-delay:300ms]" />
          </span>
        )}
      </div>
      <div className="flex items-center gap-4 text-slate-600">
        {version && <span>v{version}</span>}
        {time && (
          <span className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            {time}
          </span>
        )}
      </div>
    </footer>
  );
}
