import type { IPReport, RiskFlag, Scalar, ScoreValue } from './types';

export function cleanScalar(value: Scalar): string | null {
  if (value === null || value === undefined) return null;
  const text = String(value).trim();
  return text && text.toLowerCase() !== 'null' ? text : null;
}

export function scoreToNumber(value: ScoreValue): number | null {
  const text = cleanScalar(value);
  if (!text) return null;
  const numeric = Number(text.replace('%', ''));
  if (!Number.isFinite(numeric)) return null;
  return Math.min(100, Math.max(0, Math.round(numeric)));
}

export function normalizeRiskFlag(value: RiskFlag): boolean | null {
  if (typeof value === 'boolean') return value;
  if (typeof value === 'number') return value > 0;
  if (typeof value !== 'string') return null;

  const normalized = value.trim().toLowerCase();
  if (!normalized || normalized === 'null' || normalized === 'unknown' || normalized === 'n/a') {
    return null;
  }
  if (['true', 'yes', 'y', '1', 'risk', 'blocked', 'block'].includes(normalized)) return true;
  if (['false', 'no', 'n', '0', 'clean', 'ok'].includes(normalized)) return false;
  return null;
}

export function normalizePassFlag(value: RiskFlag): boolean | null {
  if (typeof value === 'boolean') return value;
  if (typeof value === 'number') return value > 0;
  if (typeof value !== 'string') return null;

  const normalized = value.trim().toLowerCase();
  if (!normalized || normalized === 'null' || normalized === 'unknown' || normalized === 'n/a') {
    return null;
  }
  if (['true', 'yes', 'y', '1', 'ok', 'pass', 'passed', 'open'].includes(normalized)) return true;
  if (['false', 'no', 'n', '0', 'fail', 'failed', 'blocked', 'closed'].includes(normalized)) return false;
  return null;
}

export function parseReport(jsonStr: string): IPReport {
  const parsed = JSON.parse(jsonStr) as Partial<IPReport>;
  if (!parsed || typeof parsed !== 'object') {
    throw new Error('INVALID_REPORT');
  }
  if (!parsed.Head?.IP || !parsed.Info || !parsed.Type || !parsed.Score || !parsed.Factor) {
    throw new Error('INCOMPLETE_REPORT');
  }
  if (!parsed.Media || !parsed.Mail) {
    throw new Error('INCOMPLETE_REPORT');
  }
  return parsed as IPReport;
}

export function getUserError(err: unknown): string {
  const msg = String(err);
  if (msg.includes('INVALID_REPORT') || msg.includes('INCOMPLETE_REPORT') || msg.includes('JSON')) {
    return '检测结果格式异常';
  }
  if (msg.includes('bash')) return '未找到运行环境';
  if (msg.includes('timeout') || msg.includes('Timeout') || msg.includes('timed out')) {
    return '检测超时，请稍后重试';
  }
  if (msg.includes('network') || msg.includes('connect') || msg.includes('Request failed')) {
    return '网络连接失败';
  }
  return '检测失败，请稍后重试';
}
