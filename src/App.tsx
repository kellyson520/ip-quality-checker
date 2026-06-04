import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ShieldCheck, Zap, ArrowRight } from 'lucide-react';
import type { IPReport } from './types';
import Header from './components/Header';
import IPInfoCard from './components/IPInfoCard';
import RiskScore from './components/RiskScore';
import RiskFactors from './components/RiskFactors';
import StreamingStatus from './components/StreamingStatus';
import MailStatus from './components/MailStatus';
import LoadingSpinner from './components/LoadingSpinner';
import StatusBar from './components/StatusBar';

type AppStatus = 'idle' | 'loading' | 'done' | 'error';

export default function App() {
  const [data, setData] = useState<IPReport | null>(null);
  const [status, setStatus] = useState<AppStatus>('idle');
  const [error, setError] = useState<string>('');

  const runCheck = useCallback(async () => {
    setStatus('loading');
    setError('');
    try {
      const jsonStr = await invoke<string>('run_ip_check');
      const result: IPReport = JSON.parse(jsonStr);
      setData(result);
      setStatus('done');
    } catch (err) {
      const msg = String(err);
      // User-friendly error: hide internal details, show actionable message
      if (msg.includes('bash')) {
        setError('未找到运行环境，请确保已安装 bash');
      } else if (msg.includes('timeout') || msg.includes('Timeout')) {
        setError('检测超时，请检查网络连接后重试');
      } else if (msg.includes('network') || msg.includes('Network') || msg.includes('connect')) {
        setError('网络连接失败，请检查网络后重试');
      } else {
        setError('检测失败，请稍后重试');
      }
      setStatus('error');
    }
  }, []);

  const isIdle = status === 'idle';
  const isLoading = status === 'loading';

  return (
    <div className="flex flex-col h-screen overflow-hidden">
      <Header
        onRun={runCheck}
        loading={isLoading}
        ip={data?.Head.IP}
      />

      <main className="flex-1 overflow-y-auto px-4 md:px-6 py-6">
        {isIdle && !data && (
          <div className="flex flex-col items-center justify-center h-full gap-8 animate-fade-in">
            <div className="w-24 h-24 rounded-2xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center animate-pulse-glow">
              <ShieldCheck className="w-12 h-12 text-white" />
            </div>
            <div className="text-center space-y-2">
              <h2 className="text-2xl font-bold gradient-text">IP 质量检测工具</h2>
              <p className="text-slate-500 max-w-md">
                检测您的 IP 地址质量，包括风险评分、代理/VPN 检测、流媒体解锁状态和邮件服务连通性
              </p>
            </div>
            <button onClick={runCheck} className="btn-primary text-lg px-10 py-4 flex items-center gap-3 animate-pulse-glow">
              <Zap className="w-5 h-5" />
              开始检测
              <ArrowRight className="w-5 h-5" />
            </button>
            <div className="flex items-center gap-6 text-xs text-slate-600 mt-4">
              <span className="flex items-center gap-1.5">🔍 代理检测</span>
              <span className="flex items-center gap-1.5">📺 流媒体状态</span>
              <span className="flex items-center gap-1.5">📧 邮件服务</span>
              <span className="flex items-center gap-1.5">🛡️ 风险评分</span>
            </div>
          </div>
        )}

        {isLoading && <LoadingSpinner />}

        {status === 'error' && (
          <div className="flex flex-col items-center justify-center h-full gap-4 animate-fade-in">
            <div className="w-16 h-16 rounded-full bg-red-500/20 flex items-center justify-center">
              <span className="text-3xl">⚠️</span>
            </div>
            <p className="text-red-400 text-sm max-w-md text-center">{error || '检测过程中发生错误，请重试'}</p>
            <button onClick={runCheck} className="btn-primary text-sm">重新检测</button>
          </div>
        )}

        {data && (
          <div className="max-w-5xl mx-auto space-y-5">
            <IPInfoCard head={data.Head} info={data.Info} type={data.Type} />
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
              <RiskScore score={data.Score} />
              <RiskFactors factor={data.Factor} />
            </div>
            <StreamingStatus media={data.Media} />
            <MailStatus mail={data.Mail} />
          </div>
        )}
      </main>

      <StatusBar
        status={status}
        version={data?.Head.Version}
        time={data?.Head.Time}
      />
    </div>
  );
}
