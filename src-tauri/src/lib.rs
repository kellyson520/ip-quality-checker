#[cfg(mobile)]
use serde_json::Value;
#[cfg(desktop)]
use std::io::Write;
#[cfg(desktop)]
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
        Ok("bash".to_string())
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

        if let Ok(output) = Command::new("wsl")
            .arg("bash")
            .arg("-c")
            .arg("echo ok")
            .output()
        {
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
    run_script_blocking(vec!["-j".into(), "-n".into(), "-y".into()]).await
}

/// Run IP check via bash script with custom args (desktop)
/// Args are validated against a whitelist to prevent command injection
#[cfg(desktop)]
#[tauri::command]
async fn run_ip_check_with_args(args: Vec<String>) -> Result<String, String> {
    validate_args(&args)?;
    run_script_blocking(args).await
}

#[cfg(desktop)]
async fn run_script_blocking(args: Vec<String>) -> Result<String, String> {
    let timeout = std::time::Duration::from_secs(30);
    tokio::time::timeout(timeout, tauri::async_runtime::spawn_blocking(move || run_script(args)))
        .await
        .map_err(|_| "检测超时（30秒）".to_string())?
        .map_err(|e| format!("检测任务异常: {}", e))?
}

#[cfg(desktop)]
fn validate_args(args: &[String]) -> Result<(), String> {
    for arg in args {
        if !ALLOWED_ARGS.contains(&arg.as_str()) {
            return Err(format!(
                "不允许的参数: '{}'。安全参数: {:?}",
                arg, ALLOWED_ARGS
            ));
        }
    }
    Ok(())
}

#[cfg(desktop)]
fn run_script(args: Vec<String>) -> Result<String, String> {
    let bash = find_bash()?;
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
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

/// Helper: get JSON value from a nested path.
#[cfg(mobile)]
fn jvalue<'a>(v: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for key in path {
        cur = cur.get(key)?;
    }
    Some(cur)
}

/// Helper: get non-empty string from nested JSON path.
#[cfg(mobile)]
fn opt_str(v: &Value, path: &[&str]) -> Option<String> {
    jvalue(v, path)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "null")
        .map(ToOwned::to_owned)
}

/// Helper: get number from nested JSON path.
#[cfg(mobile)]
fn opt_f64(v: &Value, path: &[&str]) -> Option<f64> {
    jvalue(v, path).and_then(|cur| cur.as_f64().or_else(|| cur.as_str()?.parse::<f64>().ok()))
}

/// Helper: get bool from nested JSON path.
#[cfg(mobile)]
fn opt_bool(v: &Value, path: &[&str]) -> Option<bool> {
    jvalue(v, path).and_then(|cur| {
        cur.as_bool()
            .or_else(|| match cur.as_str()?.to_ascii_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            })
    })
}

#[cfg(mobile)]
fn string_or_null(value: Option<String>) -> Value {
    value.map(Value::String).unwrap_or(Value::Null)
}

#[cfg(mobile)]
fn number_string_or_null(value: Option<f64>) -> Value {
    value
        .map(|v| Value::String(format!("{}", v as u32)))
        .unwrap_or(Value::Null)
}

#[cfg(mobile)]
fn bool_or_null(value: Option<bool>) -> Value {
    value.map(Value::Bool).unwrap_or(Value::Null)
}

#[cfg(mobile)]
fn any_bool_or_null(values: &[Option<bool>]) -> Value {
    if values.iter().any(|v| *v == Some(true)) {
        Value::Bool(true)
    } else if values.iter().all(|v| *v == Some(false)) {
        Value::Bool(false)
    } else {
        Value::Null
    }
}

#[cfg(mobile)]
fn region_or_null(value: Option<String>) -> Value {
    string_or_null(value.map(|s| s.trim_matches(['[', ']']).to_string()))
}

#[cfg(mobile)]
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36";

#[cfg(mobile)]
const RAW_GITHUB: &str = "https://testingcf.jsdelivr.net/gh/xykt/IPQuality@main/";

#[cfg(mobile)]
const IPQUALITY_COMMAND: &str = "bash <(curl -sL https://Check.Place) -EI";

#[cfg(mobile)]
const IPQUALITY_GITHUB: &str = "https://github.com/xykt/IPQuality";

#[cfg(mobile)]
const IPQUALITY_VERSION: &str = "v2026-03-13";

#[cfg(mobile)]
fn empty_string() -> Value {
    Value::String(String::new())
}

#[cfg(mobile)]
fn media_failed() -> (Value, Value, Value) {
    (
        Value::String("Failed".into()),
        empty_string(),
        empty_string(),
    )
}

#[cfg(mobile)]
fn media_block() -> (Value, Value, Value) {
    (
        Value::String("Block".into()),
        empty_string(),
        empty_string(),
    )
}

#[cfg(mobile)]
fn media_type(native: bool) -> Value {
    Value::String(if native { "Native" } else { "ViaDNS" }.into())
}

#[cfg(mobile)]
fn is_public_dns_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.octets()[0] == 0)
        }
        std::net::IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_multicast()
                || v6.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80)
        }
    }
}

#[cfg(mobile)]
async fn resolve_ips(host: &str) -> Vec<std::net::IpAddr> {
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::lookup_host((host, 80)),
    )
    .await
    {
        Ok(Ok(addrs)) => {
            let mut ips = Vec::new();
            for addr in addrs {
                let ip = addr.ip();
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
            ips
        }
        _ => Vec::new(),
    }
}

#[cfg(mobile)]
async fn dns_primary_ok(host: &str) -> bool {
    resolve_ips(host).await.into_iter().any(is_public_dns_ip)
}

#[cfg(mobile)]
async fn dns_answer_count_gt_two(host: &str) -> bool {
    resolve_ips(host).await.len() > 2
}

#[cfg(mobile)]
async fn dns_random_subdomain_has_no_answer(host: &str) -> bool {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    resolve_ips(&format!("test{}.{}", nanos, host))
        .await
        .is_empty()
}

#[cfg(mobile)]
async fn unlock_type_for(host: &str, use_answer_count: bool) -> Value {
    let primary = dns_primary_ok(host);
    let random = dns_random_subdomain_has_no_answer(host);
    if use_answer_count {
        let answer_count = dns_answer_count_gt_two(host);
        let (primary, answer_count, random) = tokio::join!(primary, answer_count, random);
        media_type(primary && answer_count && random)
    } else {
        let (primary, random) = tokio::join!(primary, random);
        media_type(primary && random)
    }
}

/// Fetch JSON from URL using shared client (mobile only)
#[cfg(mobile)]
async fn fetch_json(url: &str) -> Result<Value, String> {
    let resp = get_client()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))?;

    // Check if response is HTML (Cloudflare block)
    if text.starts_with("<!DOCTYPE") || text.starts_with("<html") {
        return Err("Blocked by Cloudflare".to_string());
    }

    serde_json::from_str(&text).map_err(|e| {
        format!(
            "JSON parse error: {} | body: {}",
            e,
            &text[..200.min(text.len())]
        )
    })
}

/// Fetch maxmind data with fallback (mobile only)
#[cfg(mobile)]
async fn fetch_maxmind(ip: &str) -> Value {
    // Primary: ipinfo.check.place (带中文)
    let primary_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    if let Ok(data) = fetch_json(&primary_url).await {
        if !data.is_null() && data.is_object() {
            eprintln!("[maxmind] primary OK");
            return data;
        }
    }
    eprintln!("[maxmind] primary failed, trying fallback 1");

    // Fallback 1: ipinfo.check.place (英文)
    let fallback1_url = format!("https://ipinfo.check.place/{}?lang=en", ip);
    if let Ok(data) = fetch_json(&fallback1_url).await {
        if !data.is_null() && data.is_object() {
            eprintln!("[maxmind] fallback1 OK");
            return data;
        }
    }
    eprintln!("[maxmind] fallback1 failed, trying ipapi.co");

    // Fallback 2: ipapi.co (免费 API，自动转换格式)
    let fallback2_url = format!("https://ipapi.co/{}/json/", ip);
    match fetch_json(&fallback2_url).await {
        Ok(data) if !data.is_null() && data.is_object() => {
            eprintln!(
                "[maxmind] ipapi.co OK: city={}",
                data["city"].as_str().unwrap_or("?")
            );
            let asn_str = data["asn"].as_str().unwrap_or("");
            let asn_num = asn_str.replace("AS", "").parse::<u64>().unwrap_or(0);
            serde_json::json!({
                "ASN": {
                    "AutonomousSystemNumber": asn_num,
                    "AutonomousSystemOrganization": data["org"].as_str().unwrap_or("")
                },
                "City": {
                    "Name": data["city"].as_str().unwrap_or(""),
                    "PostalCode": data["postal"].as_str().unwrap_or(""),
                    "Latitude": data["latitude"].as_f64().unwrap_or(0.0),
                    "Longitude": data["longitude"].as_f64().unwrap_or(0.0),
                    "AccuracyRadius": 0,
                    "Continent": {
                        "Code": data["continent_code"].as_str().unwrap_or(""),
                        "Name": ""
                    },
                    "Country": {
                        "IsoCode": data["country_code"].as_str().unwrap_or(""),
                        "Name": data["country_name"].as_str().unwrap_or("")
                    },
                    "Subdivisions": [{
                        "IsoCode": data["region_code"].as_str().unwrap_or(""),
                        "Name": data["region"].as_str().unwrap_or("")
                    }],
                    "Location": {
                        "TimeZone": data["timezone"].as_str().unwrap_or("")
                    }
                },
                "Country": {
                    "IsoCode": data["country_code"].as_str().unwrap_or(""),
                    "Name": data["country_name"].as_str().unwrap_or(""),
                    "RegisteredCountry": {
                        "IsoCode": data["country_code"].as_str().unwrap_or(""),
                        "Name": data["country_name"].as_str().unwrap_or("")
                    }
                }
            })
        }
        _ => {
            eprintln!("[maxmind] ALL APIs failed, returning empty");
            serde_json::json!({})
