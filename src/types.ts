export type Scalar = string | number | boolean | null | undefined;
export type ScoreValue = string | number | null | undefined;
export type RiskFlag = boolean | string | number | null | undefined;

export interface IPReport {
  Head: {
    IP: string;
    Time: string;
    Version: string;
  };
  Info: {
    ASN: string;
    Organization: string;
    Latitude: string;
    Longitude: string;
    DMS?: string;
    Map?: string;
    TimeZone: string;
    City: {
      Name: string;
      PostalCode?: string;
      SubCode?: string;
      Subdivisions?: string;
    };
    Region: { Code: string; Name: string };
    Continent: { Code: string; Name: string };
    RegisteredRegion?: { Code: string; Name: string };
    Type: string;
  };
  Type: {
    Usage?: Record<string, Scalar>;
    Company?: Record<string, Scalar>;
    Proxy?: RiskFlag;
    VPN?: RiskFlag;
    Tor?: RiskFlag;
  };
  Score: Record<string, ScoreValue>;
  Factor: {
    CountryCode?: Record<string, RiskFlag>;
    Proxy?: Record<string, RiskFlag>;
    Tor?: Record<string, RiskFlag>;
    VPN?: Record<string, RiskFlag>;
    Server?: Record<string, RiskFlag>;
    Abuser?: Record<string, RiskFlag>;
    Robot?: Record<string, RiskFlag>;
  };
  Media: Record<string, {
    Status?: Scalar;
    Result?: Scalar;
    Region?: Scalar;
    Type?: Scalar;
  }>;
  Mail: {
    Port25?: RiskFlag;
    DNSBlacklist?: {
      Total?: number | null;
      Clean?: number | null;
      Marked?: number | null;
      Blacklisted?: number | null;
    };
    [service: string]: unknown;
  };
}
