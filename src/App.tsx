import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { IPReport } from './types';
import Header from './components/Header';
import IPOverview from './components/IPOverview';
import ScoreSection from './components/ScoreSection';
import FactorSection from './components/FactorSection';
import StreamingSection from './components/StreamingSection';
import MailSection from './components/MailSection';

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
      if (msg.includes('bash')) setError('未找到运行环境');
      else if (msg.includes('timeout') || msg.includes('Timeout')) setError('检测超时');
      else if (msg.includes('network') || msg.includes('connect')) setError('网络连接失败');
      else setError('检测失败，请稍后重试');
      setStatus('error');
    }
  }, []);

  return (
    <div className="min-h-screen flex flex-col">
      <Header onRun={runCheck} loading={status === 'loading'} ip={data?.Head.IP} />

      <main className="flex-1 max-w-[960px] w-full mx-auto px-5 py-6">
        {status === 'idle' && !data && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-6 animate-fade-in">
            <h2 className="text-2xl font-semibold text-white">IP Quality Check</h2>
            <p className="text-[#666] text-sm max-w-sm text-center leading-relaxed">
              检测 IP 地址的代理/VPN 使用情况、风险评分和流媒体解锁状态
            </p>
            <button onClick={runCheck} className="btn-run mt-2">
              开始检测
            </button>
          </div>
        )}

        {status === 'loading' && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-4 animate-fade-in">
            <div className="w-5 h-5 border-2 border-[#333] border-t-white rounded-full animate-spin" />
            <p className="text-[#666] text-sm">正在检测...</p>
          </div>
        )}

        {status === 'error' && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-4 animate-fade-in">
            <p className="text-[#f87171] text-sm">{error}</p>
            <button onClick={runCheck} className="btn-run">重试</button>
          </div>
        )}

        {data && status === 'done' && (
          <div className="space-y-4 animate-fade-in">
            <IPOverview head={data.Head} info={data.Info} type={data.Type} />
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <ScoreSection score={data.Score} />
              <FactorSection factor={data.Factor} />
            </div>
            <StreamingSection media={data.Media} />
            <MailSection mail={data.Mail} />
          </div>
        )}
      </main>

      <footer className="flex items-center justify-between px-5 py-3 border-t border-[#2a2a2a] text-[11px] text-[#444] max-w-[960px] w-full mx-auto">
        <span>{status === 'done' ? '检测完成' : status === 'loading' ? '检测中...' : '就绪'}</span>
        {data && <span>{data.Head.Time}</span>}
      </footer>
    </div>
  );
}
