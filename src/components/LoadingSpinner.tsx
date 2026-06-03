import { Loader2 } from 'lucide-react';

export default function LoadingSpinner({ text }: { text?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-20 gap-6 animate-fade-in">
      <div className="relative">
        <div className="w-20 h-20 rounded-full bg-gradient-to-r from-blue-500 to-cyan-500 animate-spin-slow flex items-center justify-center">
          <div className="w-16 h-16 rounded-full bg-[#0f172a] flex items-center justify-center">
            <Loader2 className="w-8 h-8 text-blue-400 animate-spin" />
          </div>
        </div>
      </div>
      <p className="text-slate-400 text-sm">{text || '正在检测中，请稍候...'}</p>
    </div>
  );
}
