import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { IPReport } from './types';
import { getUserError, parseReport } from './report';
import Header from './components/Header';
import IPOverview from './components/IPOverview';
import ScoreSection from './components/ScoreSection';
import FactorSection from './components/FactorSection';
import StreamingSection from './components/StreamingSection';
import MailSection from './components/MailSection';

// ── UI string constants ──────────────────────────────────────────
const UI_TITLE = 'IP Quality Check';
const UI_SUBTITLE = '检测 IP 地址的代理/VPN 使用情况、风险评分和流媒体解锁状态';
const UI_BTN_START = '开始检测';
const UI_BTN_RETRY = '重试';
const UI_LOADING_TEXT = '正在检测...';
const UI_FOOTER_DONE = '检测完成';
const UI_FOOTER_LOADING = '检测中...';
const UI_FOOTER_READY = '就绪';

type AppStatus = 'idle' | 'loading' | 'done' | 'error';

export default function App() {
  const [data, setData] = useState<IPReport | null>(null);
  const [status, setStatus] = useState<AppStatus>('idle');
  const [error, setError] = useState<string>('');
  const [durationMs, setDurationMs] = useState<number | null>(null);

  const runCheck = useCallback(async () => {
    setStatus('loading');
    setError('');
    setDurationMs(null);
    const startedAt = performance.now();
    try {
      const jsonStr = await invoke<string>('run_ip_check');
      const result = parseReport(jsonStr);
      setData(result);
      setDurationMs(Math.round(performance.now() - startedAt));
      setStatus('done');
    } catch (err) {
      setError(getUserError(err));
      setStatus('error');
    }
  }, []);

  return (
    <div className="min-h-screen flex flex-col">
      <Header onRun={runCheck} loading={status === 'loading'} ip={data?.Head.IP} />

      <main className="flex-1 max-w-[960px] w-full mx-auto overflow-hidden px-3 sm:px-5 py-4 sm:py-6">
        {status === 'idle' && !data && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-6 animate-fade-in">
            <h2 className="text-xl sm:text-2xl font-semibold text-white">{UI_TITLE}</h2>
            <p className="text-[#666] text-sm max-w-sm text-center leading-relaxed px-4">
              {UI_SUBTITLE}
            </p>
            <button onClick={runCheck} className="btn-run mt-2">
              {UI_BTN_START}
            </button>
          </div>
        )}

        {status === 'loading' && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-4 animate-fade-in">
            <div className="w-5 h-5 border-2 border-[#333] border-t-white rounded-full animate-spin" />
            <p className="text-[#666] text-sm">{UI_LOADING_TEXT}</p>
          </div>
        )}

        {status === 'error' && (
          <div className="flex flex-col items-center justify-center h-[70vh] gap-4 animate-fade-in">
            <p className="text-[#f87171] text-sm">{error}</p>
            <button onClick={runCheck} className="btn-run">{UI_BTN_RETRY}</button>
          </div>
        )}

        {data && status === 'done' && (
          <div className="space-y-3 sm:space-y-4 animate-fade-in">
            <IPOverview head={data.Head} info={data.Info} type={data.Type} />
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-3 sm:gap-4">
              <div className="min-w-0">
                <ScoreSection score={data.Score} />
              </div>
              <div className="min-w-0">
                <FactorSection factor={data.Factor} />
              </div>
            </div>
            <StreamingSection media={data.Media} />
            <MailSection mail={data.Mail} />
          </div>
        )}
      </main>

      <footer className="app-footer flex items-center justify-between px-3 sm:px-5 pt-2.5 border-t border-[#2a2a2a] text-[10px] sm:text-[11px] text-[#444] max-w-[960px] w-full mx-auto">
        <span>
          {status === 'done'
            ? `${UI_FOOTER_DONE}${durationMs ? ` · ${(durationMs / 1000).toFixed(1)}s` : ''}`
            : status === 'loading'
              ? UI_FOOTER_LOADING
              : UI_FOOTER_READY}
        </span>
        {data && <span className="truncate ml-2">{data.Head.Time}</span>}
      </footer>
    </div>
  );
}
