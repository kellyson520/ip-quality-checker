import { describe, expect, it } from 'vitest';
import {
  cleanScalar,
  getUserError,
  normalizePassFlag,
  normalizeRiskFlag,
  parseReport,
  scoreToNumber,
} from './report';

const ipShSample = {
  Head: {
    IP: '36.235.*.*',
    Command: 'bash <(curl -sL https://Check.Place) -EI',
    GitHub: 'https://github.com/xykt/IPQuality',
    Time: '2026-01-15 09:31:25 UTC',
    Version: 'v2026-01-15',
  },
  Info: {
    ASN: '3462',
    Organization: 'Data Communication Business Group',
    Latitude: '24.0761',
    Longitude: '120.5648',
    DMS: '120°33′53″E, 24°4′34″N',
    Map: 'https://check.place/24.0761,120.5648,15,en',
    TimeZone: 'Asia/Taipei',
    City: {
      Name: 'Chang-hua',
      PostalCode: 'null',
      SubCode: 'CHA',
      Subdivisions: 'Changhua',
    },
    Region: { Code: 'TW', Name: 'Taiwan' },
    Continent: { Code: 'AS', Name: 'Asia' },
    RegisteredRegion: { Code: 'TW', Name: 'Taiwan' },
    Type: 'Geo-consistent',
  },
  Type: {
    Usage: {
      IPinfo: 'ISP',
      ipregistry: 'ISP',
      ipapi: 'ISP',
      AbuseIPDB: 'Line ISP',
      IP2LOCATION: 'Line ISP',
    },
    Company: {
      IPinfo: 'ISP',
      ipregistry: 'ISP',
      ipapi: 'ISP',
    },
  },
  Score: {
    IP2LOCATION: '0',
    SCAMALYTICS: '0',
    ipapi: '0.47%',
    AbuseIPDB: '0',
    IPQS: 'null',
    DBIP: '0',
  },
  Factor: {
    CountryCode: {
      IP2LOCATION: 'TW',
      ipapi: 'TW',
      ipregistry: 'TW',
      IPQS: 'TW',
      SCAMALYTICS: 'TW',
      ipdata: 'TW',
      IPinfo: 'TW',
      IPWHOIS: 'TW',
      DBIP: 'TW',
    },
    Proxy: {
      IP2LOCATION: false,
      ipapi: false,
      ipregistry: false,
      IPQS: false,
      SCAMALYTICS: false,
      ipdata: false,
      IPinfo: false,
      IPWHOIS: false,
      DBIP: false,
    },
    Tor: { IP2LOCATION: false, DBIP: null },
    VPN: { IP2LOCATION: false, ipdata: null },
    Server: { IP2LOCATION: false },
    Abuser: { IP2LOCATION: false },
    Robot: { IP2LOCATION: false },
  },
  Media: {
    TikTok: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    DisneyPlus: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    Netflix: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    Youtube: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    AmazonPrimeVideo: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    Reddit: { Status: 'Yes', Region: 'TW', Type: 'Native' },
    ChatGPT: { Status: 'Yes', Region: 'TW', Type: 'Native' },
  },
  Mail: {
    Port25: false,
    Gmail: false,
    Outlook: false,
    Yahoo: false,
    Apple: false,
    QQ: false,
    MailRU: false,
    AOL: false,
    GMX: false,
    MailCOM: false,
    '163': false,
    Sohu: false,
    Sina: false,
    DNSBlacklist: {
      Total: 439,
      Clean: 411,
      Marked: 28,
      Blacklisted: 0,
    },
  },
};

describe('report adapter', () => {
  it('accepts ip.sh JSON granularity', () => {
    const parsed = parseReport(JSON.stringify(ipShSample));

    expect(parsed.Head.Command).toContain('Check.Place');
    expect(parsed.Type.Usage?.IPinfo).toBe('ISP');
    expect(parsed.Factor.CountryCode?.IPWHOIS).toBe('TW');
    expect(parsed.Media.ChatGPT.Type).toBe('Native');
    expect(parsed.Mail.DNSBlacklist?.Total).toBe(439);
  });

  it('keeps ip.sh second-level key sets stable', () => {
    const parsed = parseReport(JSON.stringify(ipShSample));

    expect(Object.keys(parsed.Head).sort()).toEqual([
      'Command',
      'GitHub',
      'IP',
      'Time',
      'Version',
    ]);
    expect(Object.keys(parsed.Type.Company ?? {}).sort()).toEqual([
      'IPinfo',
      'ipapi',
      'ipregistry',
    ]);
    expect(Object.keys(parsed.Factor.Proxy ?? {}).sort()).toEqual([
      'DBIP',
      'IP2LOCATION',
      'IPQS',
      'IPWHOIS',
      'IPinfo',
      'SCAMALYTICS',
      'ipapi',
      'ipdata',
      'ipregistry',
    ]);
    expect(Object.keys(parsed.Mail).sort()).toEqual([
      '163',
      'AOL',
      'Apple',
      'DNSBlacklist',
      'GMX',
      'Gmail',
      'MailCOM',
      'MailRU',
      'Outlook',
      'Port25',
      'QQ',
      'Sina',
      'Sohu',
      'Yahoo',
    ]);
  });

  it('rejects incomplete reports before rendering', () => {
    expect(() => parseReport(JSON.stringify({ Head: { IP: '1.1.1.1' } }))).toThrow(
      'INCOMPLETE_REPORT',
    );
  });

  it('normalizes score values from ip.sh strings', () => {
    expect(scoreToNumber('0.47%')).toBe(0);
    expect(scoreToNumber('87')).toBe(87);
    expect(scoreToNumber('null')).toBeNull();
  });

  it('normalizes risk and pass flags without conflating semantics', () => {
    expect(normalizeRiskFlag('blocked')).toBe(true);
    expect(normalizeRiskFlag('clean')).toBe(false);
    expect(normalizePassFlag('open')).toBe(true);
    expect(normalizePassFlag('blocked')).toBe(false);
  });

  it('cleans null-like scalar values', () => {
    expect(cleanScalar(' null ')).toBeNull();
    expect(cleanScalar('TW')).toBe('TW');
  });

  it('maps internal errors to stable user-facing messages', () => {
    expect(getUserError(new Error('INCOMPLETE_REPORT'))).toBe('检测结果格式异常');
    expect(getUserError(new Error('Request failed'))).toBe('网络连接失败');
  });
});
