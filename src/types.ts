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
    Usage: Record<string, string>;
    Company: Record<string, string>;
  };
  Score: Record<string, string>;
  Factor: {
    CountryCode: Record<string, boolean | string | null>;
    Proxy: Record<string, boolean | null>;
    Tor: Record<string, boolean | null>;
    VPN: Record<string, boolean | null>;
    Server?: Record<string, boolean | null>;
    Abuser?: Record<string, boolean | null>;
    Robot?: Record<string, boolean | null>;
  };
  Media: Record<string, {
    Status: string;
    Region?: string;
    Type?: string;
  }>;
  Mail: {
    Port25?: boolean | null;
    DNSBlacklist?: {
      Total?: number;
      Clean?: number;
      Marked?: number;
      Blacklisted?: number;
    };
    [service: string]: unknown;
  };
}
