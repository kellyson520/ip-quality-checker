use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// Embedded IP quality check script (compiled into the binary, desktop only)
#[cfg(desktop)]
const IP_SCRIPT: &str = include_str!("../scripts/ip.sh");

/// Allowed arguments for run_ip_check_with_args (command injection prevention)
const ALLOWED_ARGS: &[&str] = &["-j", "-n", "-y", "-l", "-s", "-h", "-v", "-4", "-6"];

/// Shared HTTP client (mobile only, reused across requests)
#[cfg(mobile)]
static HTTP_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

#[cfg(mobile)]
fn get_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client")
    })
}

/// Find bash executable path (cross-platform, desktop only)
#[cfg(desktop)]
fn find_bash() -> Result<String, String> {
    #[cfg(unix)]
    {
        return Ok("bash".to_string());
    }

    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg("bash").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }

        let git_bash_paths = vec![
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for path in git_bash_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        if let Ok(output) = Command::new("wsl").arg("bash").arg("-c").arg("echo ok").output() {
            if output.status.success() {
                return Ok("wsl".to_string());
            }
        }

        return Err(
            "未找到 bash！请安装 Git for Windows (https://git-scm.com) 并确保添加到 PATH。"
                .to_string(),
        );
    }
}

/// Execute script via stdin pipe (desktop only)
#[cfg(desktop)]
fn exec_via_stdin(bash: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut cmd = if bash == "wsl" {
        let mut c = Command::new("wsl");
        c.arg("bash").arg("-s");
        c
    } else {
        let mut c = Command::new(bash);
        c.arg("-s");
        c
    };

    for arg in args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start script: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(IP_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script to stdin: {}", e))?;
    }

    child
        .wait_with_output()
        .map_err(|e| format!("Failed to execute script: {}", e))
}

/// Run IP check via bash script (desktop)
#[cfg(desktop)]
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    let bash = find_bash()?;
    let output = exec_via_stdin(&bash, &["-j", "-n", "-y"])?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "Script exited with status {}: {}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() { &stdout } else { &stderr }
        ));
    }

    Ok(stdout)
}

/// Run IP check via bash script with custom args (desktop)
/// Args are validated against a whitelist to prevent command injection
#[cfg(desktop)]
#[tauri::command]
async fn run_ip_check_with_args(args: Vec<String>) -> Result<String, String> {
    // Validate all args against whitelist (command injection prevention)
    for arg in &args {
        if !ALLOWED_ARGS.contains(&arg.as_str()) {
            return Err(format!(
                "不允许的参数: '{}'。安全参数: {:?}",
                arg, ALLOWED_ARGS
            ));
        }
    }
    let bash = find_bash()?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = exec_via_stdin(&bash, &arg_refs)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "Script exited with status {}: {}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() { &stdout } else { &stderr }
        ));
    }

    Ok(stdout)
}

/// Helper: get string from nested JSON path
#[cfg(mobile)]
fn jstr(v: &Value, path: &[&str]) -> String {
    let mut cur = v;
    for key in path {
        cur = cur.get(key).unwrap_or(&Value::Null);
    }
    cur.as_str().unwrap_or("null").to_string()
}

/// Helper: get f64 from nested JSON path
#[cfg(mobile)]
fn jf64(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for key in path {
        cur = cur.get(key).unwrap_or(&Value::Null);
    }
    cur.as_f64().unwrap_or(0.0)
}

/// Helper: get bool from nested JSON path
#[cfg(mobile)]
fn jbool(v: &Value, path: &[&str]) -> bool {
    let mut cur = v;
    for key in path {
        cur = cur.get(key).unwrap_or(&Value::Null);
    }
    cur.as_bool().unwrap_or(false)
}

/// Fetch JSON from URL using shared client (mobile only)
#[cfg(mobile)]
async fn fetch_json(url: &str) -> Result<Value, String> {
    let resp = get_client()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {} | body: {}", e, &text[..200.min(text.len())]))
}

/// Fetch text/HTTP status from URL (mobile only, for non-JSON endpoints)
#[cfg(mobile)]
async fn check_http_status(url: &str) -> u16 {
    match get_client().get(url).send().await {
        Ok(resp) => resp.status().as_u16(),
        Err(_) => 0,
    }
}

/// Format current time as human-readable string (matching bash script format)
#[cfg(mobile)]
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let is_leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days_in_month = [
        31,
        if is_leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 1u32;
    for &dim in &days_in_month {
        if remaining_days < dim as i64 {
            break;
        }
        remaining_days -= dim as i64;
        m += 1;
    }
    let d = remaining_days as u32 + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hours, minutes, seconds
    )
}

/// Native IP check using HTTP APIs (mobile - Android/iOS)
/// Output format matches the bash script (ip.sh) JSON output exactly
#[cfg(mobile)]
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    // Step 1: Get public IP (try multiple services)
    let ip = match fetch_json("https://api.ipify.org?format=json").await {
        Ok(v) => v["ip"].as_str().unwrap_or("").to_string(),
        Err(_) => {
            // Fallback
            let resp = get_client()
                .get("https://httpbin.org/ip")
                .send()
                .await
                .map_err(|e| format!("Cannot get IP: {}", e))?;
            let text = resp.text().await.unwrap_or_default();
            let v: Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));
            v["origin"]
                .as_str()
                .unwrap_or("")
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
    };
    if ip.is_empty() {
        return Err("无法获取公网IP".to_string());
    }

    // Step 2: Concurrent API requests (8 data sources like bash script)
    let info_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    let scam_url = format!("https://ipinfo.check.place/{}?db=scamalytics", ip);
    let abuse_url = format!("https://ipinfo.check.place/{}?db=abuseipdb", ip);
    let reg_url = format!("https://ipinfo.check.place/{}?db=ipregistry", ip);
    let ipapi_url = format!("https://ipinfo.check.place/{}?db=ipapi", ip);
    let ip2l_url = format!("https://ipinfo.check.place/{}?db=ip2location", ip);
    let ipdata_url = format!("https://ipinfo.check.place/{}?db=ipdata", ip);
    let ipinfo_url = format!("https://ipinfo.io/widget/demo/{}", ip);

    let (info_r, scam_r, abuse_r, reg_r, ipapi_r, ip2l_r, ipdata_r, ipinfo_r) = tokio::join!(
        fetch_json(&info_url),
        fetch_json(&scam_url),
        fetch_json(&abuse_url),
        fetch_json(&reg_url),
        fetch_json(&ipapi_url),
        fetch_json(&ip2l_url),
        fetch_json(&ipdata_url),
        fetch_json(&ipinfo_url)
    );

    let info = info_r.unwrap_or(serde_json::json!({}));
    let scam = scam_r.unwrap_or(serde_json::json!({}));
    let abuse = abuse_r.unwrap_or(serde_json::json!({}));
    let reg = reg_r.unwrap_or(serde_json::json!({}));
    let ipapi = ipapi_r.unwrap_or(serde_json::json!({}));
    let ip2l = ip2l_r.unwrap_or(serde_json::json!({}));
    let ipdata = ipdata_r.unwrap_or(serde_json::json!({}));
    let ipinfo = ipinfo_r.unwrap_or(serde_json::json!({}));

    // Step 5: Check streaming services concurrently
    let (tt, nf, dp, yt, am, rd, gp) = tokio::join!(
        check_http_status("https://www.tiktok.com/"),
        check_http_status("https://www.netflix.com/title/81280792"),
        check_http_status("https://www.disneyplus.com/"),
        check_http_status("https://www.youtube.com/"),
        check_http_status("https://www.amazon.com/gp/video/storefront"),
        check_http_status("https://www.reddit.com/"),
        check_http_status("https://chatgpt.com/")
    );
    let yn = |code: u16| -> &str { if code == 200 { "解锁" } else { "Block" } };

    // === Map API responses to bash script JSON format ===

    // Info: from ipinfo.check.place (maxmind data)
    let asn_num = info["ASN"]["AutonomousSystemNumber"].as_u64().unwrap_or(0);
    let asn = if asn_num > 0 {
        format!("{}", asn_num)
    } else {
        "null".to_string()
    };
    let org = jstr(&info, &["ASN", "AutonomousSystemOrganization"]);
    let city_name = jstr(&info, &["City", "Name"]);
    let lat = info["City"]["Latitude"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
    let lon = info["City"]["Longitude"].as_f64().map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
    let continent_code = jstr(&info, &["City", "Continent", "Code"]);
    let continent_name = jstr(&info, &["City", "Continent", "Name"]);
    let country_code = jstr(&info, &["Country", "IsoCode"]);
    let country_name = jstr(&info, &["Country", "Name"]);
    let reg_country_code = jstr(&info, &["Country", "RegisteredCountry", "IsoCode"]);
    let reg_country_name = jstr(&info, &["Country", "RegisteredCountry", "Name"]);
    let sub_code = info["City"]["Subdivisions"]
        .as_array()
        .and_then(|a| a.first())
        .map(|v| jstr(v, &["IsoCode"]))
        .unwrap_or_else(|| "N/A".to_string());
    let sub_name = info["City"]["Subdivisions"]
        .as_array()
        .and_then(|a| a.first())
        .map(|v| jstr(v, &["Name"]))
        .unwrap_or_else(|| "N/A".to_string());
    let timezone = jstr(&info, &["City", "Location", "TimeZone"]);

    // Info.Type: compare country vs registered country
    let info_type = if country_code != "null"
        && !country_code.is_empty()
        && country_code == reg_country_code
    {
        "本土IP地址".to_string()
    } else {
        "海外IP地址".to_string()
    };

    // Scamalytics
    let scam_score = jf64(&scam, &["scamalytics", "scamalytics_score"]);
    let scam_is_vpn = jbool(&scam, &["scamalytics", "scamalytics_proxy", "is_vpn"]);
    let scam_is_dc = jbool(&scam, &["scamalytics", "scamalytics_proxy", "is_datacenter"]);
    let scam_is_tor = jbool(&scam, &["external_datasources", "x4bnet", "is_tor"]);
    let scam_is_proxy = jbool(&scam, &["external_datasources", "firehol", "is_proxy"]);
    let scam_is_blacklisted = jbool(&scam, &["scamalytics", "is_blacklisted_external"]);
    let scam_country = jstr(&scam, &["external_datasources", "maxmind_geolite2", "ip_country_code"]);
    let scam_robot = jbool(&scam, &["external_datasources", "x4bnet", "is_blacklisted_spambot"])
        || jbool(&scam, &["external_datasources", "x4bnet", "is_bot_operamini"])
        || jbool(&scam, &["external_datasources", "x4bnet", "is_bot_semrush"]);

    // AbuseIPDB
    let abuse_score = jf64(&abuse, &["data", "abuseConfidenceScore"]);
    let abuse_usage = jstr(&abuse, &["data", "usageType"]);
    let abuse_is_tor = jbool(&abuse, &["data", "isTor"]);

    // ipregistry
    let reg_country = jstr(&reg, &["location", "country", "code"]);
    let reg_proxy = jbool(&reg, &["security", "is_proxy"]);
    let reg_vpn = jbool(&reg, &["security", "is_vpn"]);
    let reg_tor = jbool(&reg, &["security", "is_tor"]) || jbool(&reg, &["security", "is_tor_exit"]);
    let reg_server = jbool(&reg, &["security", "is_cloud_provider"]);
    let reg_abuser = jbool(&reg, &["security", "is_abuser"]);
    let reg_usage = jstr(&reg, &["connection", "type"]);
    let reg_company_type = jstr(&reg, &["company", "type"]);

    // ipapi
    let ipapi_country = jstr(&ipapi, &["location", "country_code"]);
    let ipapi_proxy = jbool(&ipapi, &["is_proxy"]);
    let ipapi_vpn = jbool(&ipapi, &["is_vpn"]);
    let ipapi_tor = jbool(&ipapi, &["is_tor"]);
    let ipapi_dc = jbool(&ipapi, &["is_datacenter"]);
    let ipapi_abuser = jbool(&ipapi, &["is_abuser"]);
    let ipapi_crawler = jbool(&ipapi, &["is_crawler"]);
    let ipapi_usage = jstr(&ipapi, &["asn", "type"]);
    let ipapi_company_type = jstr(&ipapi, &["company", "type"]);

    // ip2location
    let ip2l_country = jstr(&ip2l, &["country_code"]);
    let ip2l_usage = jstr(&ip2l, &["usage_type"]);

    // ipdata
    let ipdata_country = jstr(&ipdata, &["country_code"]);
    let ipdata_proxy = jbool(&ipdata, &["threat", "is_proxy"]);
    let ipdata_tor = jbool(&ipdata, &["threat", "is_tor"]);
    let ipdata_dc = jbool(&ipdata, &["threat", "is_datacenter"]);
    let ipdata_abuser = jbool(&ipdata, &["threat", "is_threat"])
        || jbool(&ipdata, &["threat", "is_known_abuser"])
        || jbool(&ipdata, &["threat", "is_known_attacker"]);

    // ipinfo.io
    let iio_country = jstr(&ipinfo, &["data", "country"]);
    let iio_proxy = jbool(&ipinfo, &["data", "privacy", "proxy"]);
    let iio_vpn = jbool(&ipinfo, &["data", "privacy", "vpn"]);
    let iio_tor = jbool(&ipinfo, &["data", "privacy", "tor"]);
    let iio_hosting = jbool(&ipinfo, &["data", "privacy", "hosting"]);
    let iio_usage = jstr(&ipinfo, &["data", "asn", "type"]);
    let iio_company_type = jstr(&ipinfo, &["data", "company", "type"]);

    // === Build unified output ===

    // Type.Usage: collect from all sources
    let mut usage_map = serde_json::Map::new();
    if iio_usage != "null" && !iio_usage.is_empty() { usage_map.insert("IPinfo".into(), Value::String(iio_usage)); }
    if reg_usage != "null" && !reg_usage.is_empty() { usage_map.insert("ipregistry".into(), Value::String(reg_usage)); }
    if ipapi_usage != "null" && !ipapi_usage.is_empty() { usage_map.insert("ipapi".into(), Value::String(ipapi_usage)); }
    if abuse_usage != "null" && !abuse_usage.is_empty() { usage_map.insert("AbuseIPDB".into(), Value::String(abuse_usage)); }
    if ip2l_usage != "null" && !ip2l_usage.is_empty() { usage_map.insert("IP2LOCATION".into(), Value::String(ip2l_usage)); }

    // Type.Company: collect from all sources
    let mut company_map = serde_json::Map::new();
    if iio_company_type != "null" && !iio_company_type.is_empty() { company_map.insert("IPinfo".into(), Value::String(iio_company_type)); }
    if reg_company_type != "null" && !reg_company_type.is_empty() { company_map.insert("ipregistry".into(), Value::String(reg_company_type)); }
    if ipapi_company_type != "null" && !ipapi_company_type.is_empty() { company_map.insert("ipapi".into(), Value::String(ipapi_company_type)); }

    // Score: weighted average of available sources
    let total_score = {
        let mut total = 0.0;
        let mut count = 0u32;
        if scam_score > 0.0 { total += scam_score; count += 1; }
        if abuse_score > 0.0 { total += abuse_score; count += 1; }
        if count > 0 { (total / count as f64) as u32 } else { 0 }
    };

    let result = serde_json::json!({
        "Head": {
            "IP": ip,
            "Time": chrono_now(),
            "Version": "mobile-native"
        },
        "Info": {
            "ASN": asn,
            "Organization": org,
            "Latitude": lat,
            "Longitude": lon,
            "DMS": "null",
            "Map": "null",
            "TimeZone": timezone,
            "City": { "Name": city_name },
            "Region": { "Code": country_code, "Name": country_name },
            "Continent": { "Code": continent_code, "Name": continent_name },
            "RegisteredRegion": { "Code": reg_country_code, "Name": reg_country_name },
            "Type": info_type
        },
        "Type": {
            "Usage": Value::Object(usage_map),
            "Company": Value::Object(company_map)
        },
        "Score": {
            "Total": format!("{}", total_score),
            "SCAMALYTICS": format!("{}", scam_score as u32),
            "AbuseIPDB": format!("{}", abuse_score as u32)
        },
        "Factor": {
            "CountryCode": {
                "maxmind": true,
                "ipregistry": reg_country != "null" && !reg_country.is_empty(),
                "ipapi": ipapi_country != "null" && !ipapi_country.is_empty(),
                "ipdata": ipdata_country != "null" && !ipdata_country.is_empty(),
                "IPinfo": iio_country != "null" && !iio_country.is_empty()
            },
            "Proxy": {
                "scamalytics": scam_is_proxy,
                "ipregistry": reg_proxy,
                "ipapi": ipapi_proxy,
                "ipdata": ipdata_proxy,
                "IPinfo": iio_proxy
            },
            "Tor": {
                "scamalytics": scam_is_tor,
                "ipregistry": reg_tor,
                "ipapi": ipapi_tor,
                "AbuseIPDB": abuse_is_tor,
                "ipdata": ipdata_tor,
                "IPinfo": iio_tor
            },
            "VPN": {
                "scamalytics": scam_is_vpn,
                "ipregistry": reg_vpn,
                "ipapi": ipapi_vpn,
                "IPinfo": iio_vpn
            },
            "Server": {
                "scamalytics": scam_is_dc,
                "ipregistry": reg_server,
                "ipapi": ipapi_dc,
                "ipdata": ipdata_dc,
                "IPinfo": iio_hosting
            },
            "Abuser": {
                "scamalytics": scam_is_blacklisted,
                "ipregistry": reg_abuser,
                "ipapi": ipapi_abuser,
                "ipdata": ipdata_abuser
            },
            "Robot": {
                "scamalytics": scam_robot,
                "ipapi": ipapi_crawler
            }
        },
        "Media": {
            "TikTok": { "Status": yn(tt) },
            "DisneyPlus": { "Status": yn(dp) },
            "Netflix": { "Status": yn(nf) },
            "Youtube": { "Status": yn(yt) },
            "AmazonPrimeVideo": { "Status": yn(am) },
            "Reddit": { "Status": yn(rd) },
            "ChatGPT": { "Status": yn(gp) }
        },
        "Mail": {
            "Port25": null,
            "DNSBlacklist": { "Total": null, "Clean": null, "Marked": null, "Blacklisted": null }
        }
    });

    serde_json::to_string(&result).map_err(|e| format!("Serialize error: {}", e))
}

/// Mobile version of run_ip_check_with_args (just calls run_ip_check)
#[cfg(mobile)]
#[tauri::command]
async fn run_ip_check_with_args(_args: Vec<String>) -> Result<String, String> {
    run_ip_check().await
}

/// Build and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_ip_check, run_ip_check_with_args])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
