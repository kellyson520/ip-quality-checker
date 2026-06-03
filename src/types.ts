export interface IPReport {
  Head: { IP: string; Time: string; Version: string };
  Info: {
    ASN: string;
    Organization: string;
    Latitude: string;
    Longitude: string;
    City: { Name: string };
    Region: { Code: string; Name: string };
    Continent: { Code: string; Name: string };
    Type: string;
  };
  Type: {
    Usage: Record<string, string>;
    Company: Record<string, string>;
  };
  Score: Record<string, string>;
  Factor: {
    CountryCode: Record<string, string>;
    Proxy: Record<string, boolean | null>;
    Tor: Record<string, boolean | null>;
    VPN: Record<string, boolean | null>;
    Abuser: Record<string, boolean | null>;
  };
  Media: {
    TikTok: { Result: string };
    DisneyPlus: { Result: string };
    Netflix: { Result: string };
    YouTube: { Result: string };
    AmazonPrime: { Result: string };
    Reddit: { Result: string };
    ChatGPT: { Result: string };
  };
  Mail: Record<string, { Status: string; Port: string }>;
}
