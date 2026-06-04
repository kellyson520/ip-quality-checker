#[cfg(mobile)]
use serde_json::Value;
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
    tauri::async_runtime::spawn_blocking(move || run_script(args))
        .await
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
            eprintln!("[maxmind] ipapi.co OK: city={}", data["city"].as_str().unwrap_or("?"));
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
        }
    }
}

/// Fetch text/HTTP status from URL (mobile only, for non-JSON endpoints)
#[cfg(mobile)]
async fn check_http_status(url: &str) -> u16 {
    match get_client()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) => resp.status().as_u16(),
        Err(_) => 0,
    }
}

/// Fetch raw text from URL (mobile only, for HTML/JSON response parsing)
#[cfg(mobile)]
async fn fetch_text(url: &str) -> Result<String, String> {
    let resp = get_client()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept-Language", "en")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    resp.text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))
}

/// Fetch text with custom headers (mobile only)
#[cfg(mobile)]
async fn fetch_text_with_headers(url: &str, headers: &[(&str, &str)]) -> Result<String, String> {
    let mut req = get_client().get(url).header("User-Agent", USER_AGENT);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    resp.text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))
}

#[cfg(mobile)]
async fn post_text_with_headers(
    url: &str,
    headers: &[(&str, &str)],
    body: String,
) -> Result<String, String> {
    let mut req = get_client().post(url).header("User-Agent", USER_AGENT);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    resp.text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))
}

#[cfg(mobile)]
async fn effective_url_contains(url: &str, needle: &str) -> bool {
    match get_client()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) => resp.url().as_str().contains(needle),
        Err(_) => false,
    }
}

#[cfg(mobile)]
async fn fetch_media_cookie_templates() -> Vec<String> {
    fetch_text(&format!("{}ref/cookies.txt", RAW_GITHUB))
        .await
        .map(|text| text.lines().map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

#[cfg(mobile)]
async fn fetch_ipregistry(ip: &str) -> Result<Value, String> {
    let html = fetch_text("https://ipregistry.co")
        .await
        .unwrap_or_default();
    let key = html
        .split("apiKey=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .filter(|value| !value.is_empty())
        .unwrap_or("sb69ksjcajfs4c");
    let url = format!("https://api.ipregistry.co/{}?hostname=true&key={}", ip, key);
    let text = fetch_text_with_headers(
        &url,
        &[
            ("authority", "api.ipregistry.co"),
            ("origin", "https://ipregistry.co"),
            ("referer", "https://ipregistry.co/"),
            ("Accept", "application/json"),
        ],
    )
    .await?;
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))
}

/// Generate DMS (Degrees, Minutes, Seconds) from latitude and longitude
#[cfg(mobile)]
fn generate_dms(lat: f64, lon: f64) -> String {
    fn convert_single(coord: f64, direction_positive: char, direction_negative: char) -> String {
        let dir = if coord >= 0.0 {
            direction_positive
        } else {
            direction_negative
        };
        let abs_coord = coord.abs();
        let degrees = abs_coord as i64;
        let fractional = abs_coord - degrees as f64;
        let minutes = (fractional * 60.0) as i64;
        let seconds = ((fractional * 60.0 - minutes as f64) * 60.0).round() as i64;
        format!("{}°{}′{}″{}", degrees, minutes, seconds, dir)
    }
    let lon_dms = convert_single(lon, 'E', 'W');
    let lat_dms = convert_single(lat, 'N', 'S');
    format!("{}, {}", lon_dms, lat_dms)
}

/// Generate map URL from latitude, longitude, and accuracy radius
#[cfg(mobile)]
fn generate_map_url(lat: f64, lon: f64, radius: f64) -> String {
    let zoom_level = if radius > 1000.0 {
        12
    } else if radius > 500.0 {
        13
    } else if radius > 250.0 {
        14
    } else {
        15
    };
    format!("https://check.place/{},{},{},zh", lat, lon, zoom_level)
}

/// TikTok region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_tiktok(_ip: &str) -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("tiktok.com", false).await;
    // Try main page first
    let body = match fetch_text("https://www.tiktok.com/").await {
        Ok(b) => b,
        Err(_) => return media_failed(),
    };
    let body = if body.contains("Please wait...") {
        fetch_text("https://www.tiktok.com/explore")
            .await
            .unwrap_or(body)
    } else {
        body
    };

    // Check for region in response
    if let Some(region) = extract_json_string_field(&body, "region") {
        return (
            Value::String("Yes".into()),
            region_or_null(Some(region)),
            unlock_type,
        );
    }

    // Try explore page with different headers
    let body2 = match fetch_text_with_headers("https://www.tiktok.com/explore", &[
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
        ("Accept-Language", "en"),
    ]).await {
        Ok(b) => b,
        Err(_) => return media_failed(),
    };

    if let Some(region) = extract_json_string_field(&body2, "region") {
        return (
            Value::String("IDC".into()),
            region_or_null(Some(region)),
            unlock_type,
        );
    }

    media_failed()
}

/// Netflix region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_netflix() -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("netflix.com", true).await;
    // Check two title URLs
    let url1 = "https://www.netflix.com/title/81280792";
    let url2 = "https://www.netflix.com/title/70143836";

    let (r1, r2) = tokio::join!(fetch_text(url1), fetch_text(url2));
    let body1 = r1.unwrap_or_default();
    let body2 = r2.unwrap_or_default();

    if body1.is_empty() || body2.is_empty() {
        return media_failed();
    }

    // Extract region from JSON
    let region1 = extract_netflix_region(&body1);
    let region2 = extract_netflix_region(&body2);
    let region = if region1.is_some() { region1 } else { region2 };

    let has_error1 = body1.contains("Oh no!");
    let has_error2 = body2.contains("Oh no!");

    if has_error1 && has_error2 {
        // Only original content available
        (
            Value::String("NF.Only".into()),
            region_or_null(region),
            unlock_type,
        )
    } else if !has_error1 || !has_error2 {
        // Full unlock
        (
            Value::String("Yes".into()),
            region_or_null(region),
            unlock_type,
        )
    } else {
        media_block()
    }
}

/// Extract region from Netflix response
#[cfg(mobile)]
fn extract_netflix_region(body: &str) -> Option<String> {
    // Pattern: "id":"XX"...  "countryName":"..."
    if let Some(start) = body.find(r#""id":""#) {
        let rest = &body[start + 6..];
        if let Some(end) = rest.find('"') {
            let id = &rest[..end];
            if id.len() == 2 && id.chars().all(|c| c.is_ascii_uppercase()) {
                return Some(id.to_string());
            }
        }
    }
    None
}

/// YouTube Premium detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_youtube() -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("www.youtube.com", false).await;
    let url = "https://www.youtube.com/premium";
    let body = match fetch_text_with_headers(url, &[
        ("Accept-Language", "en"),
        ("Cookie", "YSC=BiCUU3-5Gdk; CONSENT=YES+cb.20220301-11-p0.en+FX+700; GPS=1; VISITOR_INFO1_LIVE=4VwPMkB7W5A; PREF=tz=Asia.Shanghai"),
    ]).await {
        Ok(b) => b,
        Err(_) => return media_failed(),
    };

    if body.contains("www.google.cn") {
        return (
            Value::String("China".into()),
            Value::String("CN".into()),
            empty_string(),
        );
    }

    if body.contains("Premium is not available in your country") {
        return (
            Value::String("NoPrem.".into()),
            empty_string(),
            empty_string(),
        );
    }

    // Extract region
    let region = extract_youtube_region(&body);
    if body.contains("ad-free") {
        return (
            Value::String("Yes".into()),
            region_or_null(region),
            unlock_type,
        );
    }

    media_failed()
}

/// Extract region from YouTube response
#[cfg(mobile)]
fn extract_youtube_region(body: &str) -> Option<String> {
    if let Some(start) = body.find(r#""contentRegion":""#) {
        let rest = &body[start + 17..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Disney+ region detection (simplified - matching ip.sh key logic)
#[cfg(mobile)]
async fn detect_disney() -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("disneyplus.com", false).await;
    const AUTH: &str =
        "Bearer ZGlzbmV5JmJyb3dzZXImMS4wLjA.Cu56AgSfBTDag5NiRA81oLHkDZfu5L3CKadnefEAY84";
    let assertion_resp = match get_client()
        .post("https://disney.api.edge.bamgrid.com/devices")
        .header("User-Agent", USER_AGENT)
        .header("authorization", AUTH)
        .header("content-type", "application/json; charset=UTF-8")
        .body(r#"{"deviceFamily":"browser","applicationRuntime":"chrome","deviceProfile":"windows","attributes":{}}"#)
        .send()
        .await
    {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => return media_failed(),
    };

    let assertion_json: Value = match serde_json::from_str(&assertion_resp) {
        Ok(v) => v,
        Err(_) => return media_failed(),
    };
    let assertion = match opt_str(&assertion_json, &["assertion"]) {
        Some(v) => v,
        None => return media_failed(),
    };

    let cookie_templates = fetch_media_cookie_templates().await;
    let token_body = cookie_templates
        .first()
        .cloned()
        .unwrap_or_else(|| "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&latitude=0&longitude=0&platform=browser&subject_token=DISNEYASSERTION&subject_token_type=urn%3Abamtech%3Aparams%3Aoauth%3Atoken-type%3Adevice".to_string())
        .replace("DISNEYASSERTION", &assertion);
    let token_resp = match post_text_with_headers(
        "https://disney.api.edge.bamgrid.com/token",
        &[
            ("authorization", AUTH),
            ("content-type", "application/x-www-form-urlencoded"),
        ],
        token_body,
    )
    .await
    {
        Ok(text) => text,
        Err(_) => return media_failed(),
    };
    if token_resp.contains("forbidden-location") || token_resp.contains("403 ERROR") {
        return media_block();
    }
    let token_json: Value = match serde_json::from_str(&token_resp) {
        Ok(v) => v,
        Err(_) => return media_failed(),
    };
    let refresh_token = match opt_str(&token_json, &["refresh_token"]) {
        Some(v) => v,
        None => return media_failed(),
    };

    let graphql_body = cookie_templates
        .get(7)
        .cloned()
        .unwrap_or_else(|| r#"{"query":"mutation refreshToken($input: RefreshTokenInput!) {\n            refreshToken(refreshToken: $input) {\n                activeSession {\n                    sessionId\n                }\n            }\n        }","variables":{"input":{"refreshToken":"ILOVEDISNEY"}}}"#.to_string())
        .replace("ILOVEDISNEY", &refresh_token);
    let gql_resp = match post_text_with_headers(
        "https://disney.api.edge.bamgrid.com/graph/v1/device/graphql",
        &[(
            "authorization",
            "ZGlzbmV5JmJyb3dzZXImMS4wLjA.Cu56AgSfBTDag5NiRA81oLHkDZfu5L3CKadnefEAY84",
        )],
        graphql_body,
    )
    .await
    {
        Ok(text) => text,
        Err(_) => return media_failed(),
    };
    let gql_json: Value = match serde_json::from_str(&gql_resp) {
        Ok(v) => v,
        Err(_) => return media_failed(),
    };
    let region = opt_str(
        &gql_json,
        &["extensions", "sdk", "session", "location", "countryCode"],
    );
    let supported = opt_bool(
        &gql_json,
        &["extensions", "sdk", "session", "inSupportedLocation"],
    );
    let unavailable = effective_url_contains("https://disneyplus.com", "unavailable").await;
    match (region, supported, unavailable) {
        (Some(region), _, _) if region == "JP" => (
            Value::String("Yes".into()),
            Value::String(region),
            unlock_type,
        ),
        (Some(region), Some(false), false) => (
            Value::String("Pending".into()),
            Value::String(region),
            unlock_type,
        ),
        (Some(_), _, true) | (None, _, _) => media_block(),
        (Some(region), Some(true), _) => (
            Value::String("Yes".into()),
            Value::String(region),
            unlock_type,
        ),
        (Some(_), None, _) => media_failed(),
    }
}

/// Amazon Prime Video region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_amazon() -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("www.primevideo.com", false).await;
    let url = "https://www.primevideo.com";
    let body = match fetch_text(url).await {
        Ok(b) => b,
        Err(_) => return media_failed(),
    };

    // Extract currentTerritory
    if let Some(territory) = extract_amazon_territory(&body) {
        return (
            Value::String("Yes".into()),
            Value::String(territory),
            unlock_type,
        );
    }

    media_block()
}

/// Extract territory from Amazon response
#[cfg(mobile)]
fn extract_amazon_territory(body: &str) -> Option<String> {
    if let Some(start) = body.find(r#""currentTerritory":""#) {
        let rest = &body[start + 19..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Reddit region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_reddit() -> (Value, Value, Value) {
    let unlock_type = unlock_type_for("reddit.com", true).await;
    let url = "https://www.reddit.com/";
    let resp = match get_client()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return media_failed(),
    };

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();

    match status {
        200 => {
            let region = extract_reddit_region(&body);
            (
                Value::String("Yes".into()),
                region_or_null(region),
                unlock_type,
            )
        }
        403 => media_block(),
        _ => media_failed(),
    }
}

/// Extract region from Reddit response
#[cfg(mobile)]
fn extract_reddit_region(body: &str) -> Option<String> {
    if let Some(start) = body.find(r#"country=""#) {
        let rest = &body[start + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// ChatGPT detection (matching ip.sh complex logic)
#[cfg(mobile)]
async fn detect_chatgpt() -> (Value, Value, Value) {
    let (chat_type, ios_type, api_type) = tokio::join!(
        unlock_type_for("chat.openai.com", true),
        unlock_type_for("ios.chat.openai.com", true),
        unlock_type_for("api.openai.com", false)
    );
    let unlock_type = if chat_type == Value::String("ViaDNS".into())
        || ios_type == Value::String("ViaDNS".into())
        || api_type == Value::String("ViaDNS".into())
    {
        Value::String("ViaDNS".into())
    } else {
        Value::String("Native".into())
    };
    // Check multiple endpoints like ip.sh
    let (r1, r2, r3) = tokio::join!(
        fetch_text("https://api.openai.com/compliance/cookie_requirements"),
        fetch_text("https://ios.chat.openai.com/"),
        fetch_text("https://chat.openai.com/cdn-cgi/trace")
    );

    let body1 = r1.unwrap_or_default();
    let body2 = r2.unwrap_or_default();
    let trace = r3.unwrap_or_default();

    let mut has_unsupported = body1.contains("unsupported_country");
    let has_vpn = body2.contains("VPN");
    if has_unsupported && check_http_status("https://chatgpt.com/favicon.ico").await != 403 {
        has_unsupported = false;
    }

    // Extract country code from trace
    let country_code = extract_trace_country(&trace);

    if !has_unsupported && !has_vpn && !body1.is_empty() && !body2.is_empty() {
        (
            Value::String("Yes".into()),
            region_or_null(country_code),
            unlock_type,
        )
    } else if has_vpn && has_unsupported {
        media_block()
    } else if !has_unsupported && has_vpn {
        (
            Value::String("WebOnly".into()),
            region_or_null(country_code),
            unlock_type,
        )
    } else if has_unsupported && !has_vpn {
        (
            Value::String("APPOnly".into()),
            region_or_null(country_code),
            unlock_type,
        )
    } else {
        media_failed()
    }
}

/// Extract country from Cloudflare trace
#[cfg(mobile)]
fn extract_trace_country(trace: &str) -> Option<String> {
    for line in trace.lines() {
        if let Some(rest) = line.strip_prefix("loc=") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Extract JSON string field (for TikTok region parsing)
#[cfg(mobile)]
fn extract_json_string_field(body: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", field);
    if let Some(start) = body.find(&pattern) {
        let rest = &body[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Bilibili TV detection (simple HTTP check like ip.sh)
#[cfg(mobile)]
async fn check_bilibili() -> (Value, Value, Value) {
    let url = "https://www.bilibili.tv/";
    let status = check_http_status(url).await;
    if status == 200 {
        (
            Value::String("Yes".into()),
            Value::Null,
            Value::String("Native".into()),
        )
    } else if status == 0 {
        media_failed()
    } else {
        media_block()
    }
}

/// Check raw TCP connectivity with a 5-second timeout (mobile only)
#[cfg(mobile)]
async fn check_smtp_banner(addr: &str) -> Option<bool> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(_)) => return Some(false),
        Err(_) => return Some(false),
    };

    let mut buf = [0u8; 256];
    let read = match tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut buf))
        .await
    {
        Ok(Ok(n)) => n,
        Ok(Err(_)) => return Some(false),
        Err(_) => return Some(false),
    };
    let _ = stream.write_all(b"QUIT\r\n").await;
    let banner = String::from_utf8_lossy(&buf[..read]);
    Some(banner.contains("220"))
}

#[cfg(mobile)]
async fn resolve_mx_hosts(domain: &str) -> Option<Vec<String>> {
    let url = format!("https://dns.google/resolve?name={}&type=MX", domain);
    let data = fetch_json(&url).await.ok()?;
    let answers = data.get("Answer")?.as_array()?;
    let mut hosts = answers
        .iter()
        .filter_map(|answer| {
            let data = answer.get("data")?.as_str()?;
            let mut parts = data.split_whitespace();
            let priority = parts.next()?.parse::<u16>().ok()?;
            let host = parts.next()?.trim_end_matches('.').to_string();
            if host.is_empty() {
                None
            } else {
                Some((priority, host))
            }
        })
        .collect::<Vec<_>>();
    if hosts.is_empty() {
        return None;
    }
    hosts.sort_by_key(|(priority, _)| *priority);
    Some(hosts.into_iter().map(|(_, host)| host).collect())
}

#[cfg(mobile)]
async fn check_email_service(domain: &str) -> Value {
    let hosts = match resolve_mx_hosts(domain).await {
        Some(hosts) => hosts,
        None => return Value::Null,
    };
    let mut saw_definitive_failure = false;
    for host in hosts {
        match check_smtp_banner(&format!("{}:25", host)).await {
            Some(true) => return Value::Bool(true),
            Some(false) => saw_definitive_failure = true,
            None => {}
        }
    }
    if saw_definitive_failure {
        Value::Bool(false)
    } else {
        Value::Null
    }
}

/// Check SMTP port 25 connectivity (matching ip.sh's nc-based check)
#[cfg(mobile)]
async fn check_smtp_port25(_ip: &str) -> Value {
    match std::net::TcpListener::bind(("0.0.0.0", 25)) {
        Ok(listener) => drop(listener),
        Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => return Value::Null,
        Err(_) => {}
    }
    match check_smtp_banner("smtp.mailgun.org:25").await {
        Some(true) => Value::Bool(true),
        Some(false) => Value::Bool(false),
        None => Value::Null,
    }
}

#[cfg(mobile)]
fn reversed_ipv4(ip: &str) -> Option<String> {
    let octets = ip
        .split('.')
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if octets.len() != 4 {
        return None;
    }
    Some(format!(
        "{}.{}.{}.{}",
        octets[3], octets[2], octets[1], octets[0]
    ))
}

#[cfg(mobile)]
async fn check_dnsbl(ip: &str) -> Value {
    let reversed = match reversed_ipv4(ip) {
        Some(value) => value,
        None => {
            return serde_json::json!({
                "Total": null,
                "Clean": null,
                "Marked": null,
                "Blacklisted": null
            });
        }
    };
    let list = fetch_text(&format!("{}ref/dnsbl.list", RAW_GITHUB))
        .await
        .unwrap_or_default();
    let mut zones = list
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    zones.sort();
    zones.dedup();

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(50));
    let mut tasks = tokio::task::JoinSet::new();
    for zone in zones {
        let query = format!("{}.{}", reversed, zone);
        let semaphore = semaphore.clone();
        tasks.spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            let ips = resolve_ips(&query).await;
            if ips.is_empty() {
                "Clean"
            } else if ips
                .iter()
                .any(|ip| matches!(ip, std::net::IpAddr::V4(v4) if v4.octets() == [127, 0, 0, 2]))
            {
                "Blacklisted"
            } else {
                "Other"
            }
        });
    }

    let mut total = 0u64;
    let mut clean = 0u64;
    let mut marked = 0u64;
    let mut blacklisted = 0u64;
    while let Some(result) = tasks.join_next().await {
        if let Ok(status) = result {
            total += 1;
            match status {
                "Clean" => clean += 1,
                "Blacklisted" => blacklisted += 1,
                _ => marked += 1,
            }
        }
    }

    serde_json::json!({
        "Total": total,
        "Clean": clean,
        "Marked": marked,
        "Blacklisted": blacklisted
    })
}

/// Parse DBIP HTML to extract robot/proxy/abuser/risk (matching ip.sh's awk logic)
#[cfg(mobile)]
fn parse_dbip_html(body: &str) -> (Value, Value, Value, Value, Value) {
    if body.is_empty() {
        return (
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
        );
    }

    // Find crawler/proxy/abuser status from HTML
    let mut robot = Value::Null;
    let mut proxy = Value::Null;
    let mut abuser = Value::Null;
    let mut score = Value::Null;
    let mut country_code = Value::Null;

    // Extract country code from JSON-LD
    if let Some(start) = body.find(r#""countryCode""#) {
        let rest = &body[start + 14..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..];
            if let Some(q1) = after_colon.find('"') {
                let after_q1 = &after_colon[q1 + 1..];
                if let Some(q2) = after_q1.find('"') {
                    country_code = Value::String(after_q1[..q2].to_string());
                }
            }
        }
    }

    if let Some(start) = body.find("Estimated threat level for this IP address is") {
        let rest = &body[start..];
        if let Some(span_start) = rest.find("<span") {
            let after_span = &rest[span_start..];
            if let Some(gt) = after_span.find('>') {
                let text = &after_span[gt + 1..];
                if let Some(end) = text.find('<') {
                    score = match text[..end].trim().to_ascii_lowercase().as_str() {
                        "low" => Value::String("0".into()),
                        "medium" => Value::String("50".into()),
                        "high" => Value::String("100".into()),
                        _ => Value::Null,
                    };
                }
            }
        }
    }

    let table_start = body
        .find("<th class='text-center'>Crawler")
        .or_else(|| body.find(r#"<th class="text-center">Crawler"#));
    if let Some(start) = table_start {
        let table = &body[start..];
        let mut values = Vec::new();
        let mut rest = table;
        while let Some(pos) = rest.find("sr-only") {
            rest = &rest[pos + "sr-only".len()..];
            if let Some(gt) = rest.find('>') {
                let text = &rest[gt + 1..];
                if let Some(end) = text.find('<') {
                    let value = match text[..end].trim() {
                        "Yes" => Some(true),
                        "No" => Some(false),
                        _ => None,
                    };
                    if let Some(v) = value {
                        values.push(v);
                    }
                    if values.len() >= 3 {
                        break;
                    }
                }
            }
        }
        if let Some(v) = values.first() {
            robot = Value::Bool(*v);
        }
        if let Some(v) = values.get(1) {
            proxy = Value::Bool(*v);
        }
        if let Some(v) = values.get(2) {
            abuser = Value::Bool(*v);
        }
    }

    (robot, proxy, abuser, score, country_code)
}

/// Parse IPQS response (matching ip.sh logic)
#[cfg(mobile)]
fn parse_ipqs(data: &Value) -> (Option<f64>, Value, Value, Value, Value, Value, Value) {
    let score = opt_f64(data, &["fraud_score"]);
    let country = string_or_null(opt_str(data, &["country_code"]));
    let proxy = bool_or_null(opt_bool(data, &["proxy"]));
    let tor = bool_or_null(opt_bool(data, &["tor"]));
    let vpn = bool_or_null(opt_bool(data, &["vpn"]));
    let abuser = bool_or_null(opt_bool(data, &["recent_abuse"]));
    let robot = bool_or_null(opt_bool(data, &["bot_status"]));
    (score, country, proxy, tor, vpn, abuser, robot)
}

#[cfg(mobile)]
fn parse_ipapi_score(data: &Value) -> Option<f64> {
    opt_str(data, &["company", "abuser_score"]).and_then(|text| {
        text.split_whitespace()
            .next()
            .and_then(|raw| raw.parse::<f64>().ok())
            .map(|score| score * 100.0)
    })
}

#[cfg(mobile)]
fn score_string_or_null(value: Option<f64>) -> Value {
    match value {
        Some(v) if (v.fract()).abs() < f64::EPSILON => Value::String(format!("{}", v as i64)),
        Some(v) => Value::String(format!("{}", v)),
        None => Value::Null,
    }
}

#[cfg(mobile)]
fn percent_score_or_null(value: Option<f64>) -> Value {
    value
        .map(|v| Value::String(format!("{:.2}%", v)))
        .unwrap_or(Value::Null)
}

#[cfg(mobile)]
fn classify_standard_type(value: Option<String>) -> Value {
    let label = match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "business" => "Business",
        "isp" => "ISP",
        "hosting" => "Hosting",
        "education" => "Education",
        "government" => "Government",
        "banking" => "Banking",
        _ => "Other",
    };
    Value::String(label.into())
}

#[cfg(mobile)]
fn classify_abuse_usage(value: Option<String>) -> Value {
    let label = match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "commercial" => "Business",
        "data center/web hosting/transit" => "Hosting",
        "university/college/school" => "Education",
        "government" => "Government",
        "banking" => "Banking",
        "organization" => "Organization",
        "military" => "Military",
        "library" => "Library",
        "content delivery network" => "CDN",
        "fixed line isp" => "Line ISP",
        "mobile isp" => "Mobile ISP",
        "search engine spider" => "Web Spider",
        "reserved" => "Reserved",
        _ => "Other",
    };
    Value::String(label.into())
}

#[cfg(mobile)]
fn classify_ip2location_usage(value: Option<String>) -> Value {
    let raw = value.unwrap_or_default();
    let first = raw.split('/').next().unwrap_or("").to_ascii_uppercase();
    let label = match first.as_str() {
        "COM" => "Business",
        "DCH" => "Hosting",
        "EDU" => "Education",
        "GOV" => "Government",
        "ORG" => "Organization",
        "MIL" => "Military",
        "LIB" => "Library",
        "CDN" => "CDN",
        "ISP" => "Line ISP",
        "MOB" => "Mobile ISP",
        "SES" => "Web Spider",
        "RSV" => "Reserved",
        _ => "Other",
    };
    Value::String(label.into())
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
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
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

    // Step 2: Concurrent API requests (10 data sources matching ip.sh)
    // Use fetch_maxmind for maxmind data (with fallback)
    let scam_url = format!("https://ipinfo.check.place/{}?db=scamalytics", ip);
    let abuse_url = format!("https://ipinfo.check.place/{}?db=abuseipdb", ip);
    let ipapi_url = format!("https://api.ipapi.is/?q={}", ip);
    let ip2l_url = format!("https://ipinfo.check.place/{}?db=ip2location", ip);
    let ipdata_url = format!("https://ipinfo.check.place/{}?db=ipdata", ip);
    let ipqs_url = format!("https://ipinfo.check.place/{}?db=ipqualityscore", ip);
    let ipinfo_url = format!("https://ipinfo.io/widget/demo/{}", ip);

    // Fetch maxmind data with fallback (separate call)
    let info = fetch_maxmind(&ip).await;

    // Fetch other APIs concurrently
    let (scam_r, abuse_r, reg_r, ipapi_r, ip2l_r, ipdata_r, ipqs_r, ipinfo_r) = tokio::join!(
        fetch_json(&scam_url),
        fetch_json(&abuse_url),
        fetch_ipregistry(&ip),
        fetch_json(&ipapi_url),
        fetch_json(&ip2l_url),
        fetch_json(&ipdata_url),
        fetch_json(&ipqs_url),
        fetch_json(&ipinfo_url)
    );

    let scam = scam_r.unwrap_or(serde_json::json!({}));
    let abuse = abuse_r.unwrap_or(serde_json::json!({}));
    let reg = reg_r.unwrap_or(serde_json::json!({}));
    let ipapi = ipapi_r.unwrap_or(serde_json::json!({}));
    let ip2l = ip2l_r.unwrap_or(serde_json::json!({}));
    let ipdata = ipdata_r.unwrap_or(serde_json::json!({}));
    let ipqs = ipqs_r.unwrap_or(serde_json::json!({}));
    let ipinfo = ipinfo_r.unwrap_or(serde_json::json!({}));

    // Step 3: Fetch DBIP data (HTML scraping like ip.sh)
    let dbip_url = format!("https://db-ip.com/{}", ip);
    let dbip_body = fetch_text(&dbip_url).await.unwrap_or_default();

    // Step 5: Check streaming services with REAL region detection (matching ip.sh logic)
    let (
        (tt_status, tt_region, tt_type),
        (nf_status, nf_region, nf_type),
        (dp_status, dp_region, dp_type),
        (yt_status, yt_region, yt_type),
        (am_status, am_region, am_type),
        (rd_status, rd_region, rd_type),
        (gp_status, gp_region, gp_type),
    ) = tokio::join!(
        detect_tiktok(&ip),
        detect_netflix(),
        detect_disney(),
        detect_youtube(),
        detect_amazon(),
        detect_reddit(),
        detect_chatgpt()
    );

    // Step 6: Check mail services. ip.sh only tests providers when local port 25 works.
    let port25 = check_smtp_port25(&ip).await;
    let dnsbl = check_dnsbl(&ip).await;
    let (gmail, outlook, yahoo, apple, qq, mailru, aol, gmx, mailcom, mail163, sohu, sina) =
        if port25 == Value::Bool(true) {
            tokio::join!(
                check_email_service("gmail.com"),
                check_email_service("outlook.com"),
                check_email_service("yahoo.com"),
                check_email_service("me.com"),
                check_email_service("qq.com"),
                check_email_service("mail.ru"),
                check_email_service("aol.com"),
                check_email_service("gmx.com"),
                check_email_service("mail.com"),
                check_email_service("163.com"),
                check_email_service("sohu.com"),
                check_email_service("sina.com")
            )
        } else {
            let value = if port25.is_null() {
                Value::Null
            } else {
                Value::Bool(false)
            };
            (
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value.clone(),
                value,
            )
        };

    // === Map API responses to bash script JSON format ===

    // Info: from ipinfo.check.place (maxmind data)
    let asn = info["ASN"]["AutonomousSystemNumber"]
        .as_u64()
        .map(|v| Value::String(v.to_string()))
        .or_else(|| opt_str(&info, &["ASN", "AutonomousSystemNumber"]).map(Value::String))
        .unwrap_or(Value::Null);
    let org = string_or_null(opt_str(&info, &["ASN", "AutonomousSystemOrganization"]));
    let city_name = string_or_null(opt_str(&info, &["City", "Name"]));
    let city_postal = string_or_null(opt_str(&info, &["City", "PostalCode"]));
    let lat_val = info["City"]["Latitude"].as_f64();
    let lon_val = info["City"]["Longitude"].as_f64();
    let lat = lat_val
        .map(|v| Value::String(v.to_string()))
        .unwrap_or(Value::Null);
    let lon = lon_val
        .map(|v| Value::String(v.to_string()))
        .unwrap_or(Value::Null);
    let rad = info["City"]["AccuracyRadius"].as_f64();
    let continent_code = string_or_null(opt_str(&info, &["City", "Continent", "Code"]));
    let continent_name = string_or_null(opt_str(&info, &["City", "Continent", "Name"]));
    let country_code = opt_str(&info, &["Country", "IsoCode"]);
    let country_name = string_or_null(opt_str(&info, &["Country", "Name"]));
    let reg_country_code = opt_str(&info, &["Country", "RegisteredCountry", "IsoCode"]);
    let reg_country_name =
        string_or_null(opt_str(&info, &["Country", "RegisteredCountry", "Name"]));
    let sub_code = info["City"]["Subdivisions"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| opt_str(v, &["IsoCode"]));
    let sub_name = info["City"]["Subdivisions"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| opt_str(v, &["Name"]));
    let timezone = string_or_null(opt_str(&info, &["City", "Location", "TimeZone"]));

    // DMS and Map: calculate from lat/lon (matching ip.sh logic)
    let (dms, map_url) = if let (Some(lat_f), Some(lon_f)) = (lat_val, lon_val) {
        let dms_str = generate_dms(lat_f, lon_f);
        let map_str = generate_map_url(lat_f, lon_f, rad.unwrap_or(1001.0));
        (Value::String(dms_str), Value::String(map_str))
    } else {
        (Value::Null, Value::Null)
    };

    // Info.Type: compare country vs registered country
    let info_type = match (&country_code, &reg_country_code) {
        (Some(country), Some(registered)) if country == registered => {
            Value::String("Geo-consistent".to_string())
        }
        (Some(_), Some(_)) => Value::String("Geo-discrepant".to_string()),
        _ => Value::Null,
    };

    // Scamalytics
    let scam_score = opt_f64(&scam, &["scamalytics", "scamalytics_score"]);
    let scam_is_vpn = opt_bool(&scam, &["scamalytics", "scamalytics_proxy", "is_vpn"]);
    let scam_is_dc = opt_bool(
        &scam,
        &["scamalytics", "scamalytics_proxy", "is_datacenter"],
    );
    let scam_is_tor = opt_bool(&scam, &["external_datasources", "x4bnet", "is_tor"]);
    let scam_is_proxy = opt_bool(&scam, &["external_datasources", "firehol", "is_proxy"]);
    let scam_is_blacklisted = opt_bool(&scam, &["scamalytics", "is_blacklisted_external"]);
    let scam_country = opt_str(
        &scam,
        &[
            "external_datasources",
            "maxmind_geolite2",
            "ip_country_code",
        ],
    );
    let scam_robot = any_bool_or_null(&[
        opt_bool(
            &scam,
            &["external_datasources", "x4bnet", "is_blacklisted_spambot"],
        ),
        opt_bool(
            &scam,
            &["external_datasources", "x4bnet", "is_bot_operamini"],
        ),
        opt_bool(&scam, &["external_datasources", "x4bnet", "is_bot_semrush"]),
    ]);

    // AbuseIPDB
    let abuse_score = opt_f64(&abuse, &["data", "abuseConfidenceScore"]);
    let abuse_usage = opt_str(&abuse, &["data", "usageType"]);
    let abuse_is_tor = opt_bool(&abuse, &["data", "isTor"]);

    // ipregistry
    let reg_country = opt_str(&reg, &["location", "country", "code"]);
    let reg_proxy = opt_bool(&reg, &["security", "is_proxy"]);
    let reg_vpn = opt_bool(&reg, &["security", "is_vpn"]);
    let reg_tor = match (
        opt_bool(&reg, &["security", "is_tor"]),
        opt_bool(&reg, &["security", "is_tor_exit"]),
    ) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    };
    let reg_server = opt_bool(&reg, &["security", "is_cloud_provider"]);
    let reg_abuser = opt_bool(&reg, &["security", "is_abuser"]);
    let reg_usage = opt_str(&reg, &["connection", "type"]);
    let reg_company_type = opt_str(&reg, &["company", "type"]);

    // ipapi
    let ipapi_country = opt_str(&ipapi, &["location", "country_code"]);
    let ipapi_score = parse_ipapi_score(&ipapi);
    let ipapi_proxy = opt_bool(&ipapi, &["is_proxy"]);
    let ipapi_vpn = opt_bool(&ipapi, &["is_vpn"]);
    let ipapi_tor = opt_bool(&ipapi, &["is_tor"]);
    let ipapi_dc = opt_bool(&ipapi, &["is_datacenter"]);
    let ipapi_abuser = opt_bool(&ipapi, &["is_abuser"]);
    let ipapi_crawler = opt_bool(&ipapi, &["is_crawler"]);
    let ipapi_usage = opt_str(&ipapi, &["asn", "type"]);
    let ipapi_company_type = opt_str(&ipapi, &["company", "type"]);

    // ip2location
    let ip2l_country = opt_str(&ip2l, &["country_code"]);
    let ip2l_usage = opt_str(&ip2l, &["usage_type"]);
    let ip2l_company_type = opt_str(&ip2l, &["as_info", "as_usage_type"]);
    let ip2l_score = opt_f64(&ip2l, &["fraud_score"]);
    let ip2l_proxy = match (
        opt_bool(&ip2l, &["is_proxy"]),
        opt_bool(&ip2l, &["proxy", "is_public_proxy"]),
        opt_bool(&ip2l, &["proxy", "is_web_proxy"]),
    ) {
        (Some(true), _, _) | (_, Some(true), _) | (_, _, Some(true)) => Some(true),
        (Some(false), Some(false), Some(false)) => Some(false),
        _ => None,
    };
    let ip2l_tor = opt_bool(&ip2l, &["proxy", "is_tor"]);
    let ip2l_vpn = opt_bool(&ip2l, &["proxy", "is_vpn"]);
    let ip2l_server = opt_bool(&ip2l, &["proxy", "is_data_center"]);
    let ip2l_abuser = opt_bool(&ip2l, &["proxy", "is_spammer"]);
    let ip2l_robot = match (
        opt_bool(&ip2l, &["proxy", "is_web_crawler"]),
        opt_bool(&ip2l, &["proxy", "is_scanner"]),
        opt_bool(&ip2l, &["proxy", "is_botnet"]),
    ) {
        (Some(true), _, _) | (_, Some(true), _) | (_, _, Some(true)) => Some(true),
        (Some(false), Some(false), Some(false)) => Some(false),
        _ => None,
    };

    // ipdata
    let ipdata_country = opt_str(&ipdata, &["country_code"]);
    let ipdata_proxy = opt_bool(&ipdata, &["threat", "is_proxy"]);
    let ipdata_tor = opt_bool(&ipdata, &["threat", "is_tor"]);
    let ipdata_dc = opt_bool(&ipdata, &["threat", "is_datacenter"]);
    let ipdata_abuser = any_bool_or_null(&[
        opt_bool(&ipdata, &["threat", "is_threat"]),
        opt_bool(&ipdata, &["threat", "is_known_abuser"]),
        opt_bool(&ipdata, &["threat", "is_known_attacker"]),
    ]);

    // ipinfo.io
    let iio_country = opt_str(&ipinfo, &["data", "country"]);
    let iio_proxy = opt_bool(&ipinfo, &["data", "privacy", "proxy"]);
    let iio_vpn = opt_bool(&ipinfo, &["data", "privacy", "vpn"]);
    let iio_tor = opt_bool(&ipinfo, &["data", "privacy", "tor"]);
    let iio_hosting = opt_bool(&ipinfo, &["data", "privacy", "hosting"]);
    let iio_usage = opt_str(&ipinfo, &["data", "asn", "type"]);
    let iio_company_type = opt_str(&ipinfo, &["data", "company", "type"]);

    // IPQS (ipqualityscore)
    let (ipqs_score, ipqs_country, ipqs_proxy, ipqs_tor, ipqs_vpn, ipqs_abuser, ipqs_robot) =
        parse_ipqs(&ipqs);

    // DBIP (parse HTML)
    let (dbip_robot, dbip_proxy, dbip_abuser, dbip_risk, dbip_country) =
        parse_dbip_html(&dbip_body);

    // === Build unified output ===
    let usage_map = serde_json::json!({
        "IPinfo": classify_standard_type(iio_usage),
        "ipregistry": classify_standard_type(reg_usage),
        "ipapi": classify_standard_type(ipapi_usage),
        "AbuseIPDB": classify_abuse_usage(abuse_usage),
        "IP2LOCATION": classify_ip2location_usage(ip2l_usage)
    });

    let company_map = serde_json::json!({
        "IPinfo": classify_standard_type(iio_company_type),
        "ipregistry": classify_standard_type(reg_company_type),
        "ipapi": classify_standard_type(ipapi_company_type),
        "IP2LOCATION": classify_ip2location_usage(ip2l_company_type)
    });

    let result = serde_json::json!({
        "Head": {
            "IP": ip,
            "Command": Value::Null,
            "GitHub": Value::Null,
            "Time": chrono_now(),
            "Version": "mobile-native"
        },
        "Info": {
            "ASN": asn,
            "Organization": org,
            "Latitude": lat,
            "Longitude": lon,
            "DMS": dms,
            "Map": map_url,
            "TimeZone": timezone,
            "City": {
                "Name": city_name,
                "PostalCode": city_postal,
                "SubCode": string_or_null(sub_code),
                "Subdivisions": string_or_null(sub_name)
            },
            "Region": { "Code": string_or_null(country_code.clone()), "Name": country_name },
            "Continent": { "Code": continent_code, "Name": continent_name },
            "RegisteredRegion": { "Code": string_or_null(reg_country_code.clone()), "Name": reg_country_name },
            "Type": info_type
        },
        "Type": {
            "Usage": usage_map,
            "Company": company_map
        },
        "Score": {
            "IP2LOCATION": score_string_or_null(ip2l_score),
            "SCAMALYTICS": score_string_or_null(scam_score),
            "ipapi": percent_score_or_null(ipapi_score),
            "AbuseIPDB": score_string_or_null(abuse_score),
            "IPQS": score_string_or_null(ipqs_score),
            "DBIP": dbip_risk
        },
        "Factor": {
            "CountryCode": {
                "IP2LOCATION": string_or_null(ip2l_country),
                "ipapi": string_or_null(ipapi_country),
                "ipregistry": string_or_null(reg_country),
                "IPQS": ipqs_country,
                "SCAMALYTICS": string_or_null(scam_country),
                "ipdata": string_or_null(ipdata_country),
                "IPinfo": string_or_null(iio_country),
                "IPWHOIS": Value::Null,
                "DBIP": dbip_country
            },
            "Proxy": {
                "IP2LOCATION": bool_or_null(ip2l_proxy),
                "ipapi": bool_or_null(ipapi_proxy),
                "ipregistry": bool_or_null(reg_proxy),
                "IPQS": ipqs_proxy,
                "SCAMALYTICS": bool_or_null(scam_is_proxy),
                "ipdata": bool_or_null(ipdata_proxy),
                "IPinfo": bool_or_null(iio_proxy),
                "IPWHOIS": Value::Null,
                "DBIP": dbip_proxy
            },
            "Tor": {
                "IP2LOCATION": bool_or_null(ip2l_tor),
                "ipapi": bool_or_null(ipapi_tor),
                "ipregistry": bool_or_null(reg_tor),
                "IPQS": ipqs_tor,
                "SCAMALYTICS": bool_or_null(scam_is_tor),
                "ipdata": bool_or_null(ipdata_tor),
                "IPinfo": bool_or_null(iio_tor),
                "IPWHOIS": Value::Null,
                "DBIP": Value::Null
            },
            "VPN": {
                "IP2LOCATION": bool_or_null(ip2l_vpn),
                "ipapi": bool_or_null(ipapi_vpn),
                "ipregistry": bool_or_null(reg_vpn),
                "IPQS": ipqs_vpn,
                "SCAMALYTICS": bool_or_null(scam_is_vpn),
                "ipdata": Value::Null,
                "IPinfo": bool_or_null(iio_vpn),
                "IPWHOIS": Value::Null,
                "DBIP": Value::Null
            },
            "Server": {
                "IP2LOCATION": bool_or_null(ip2l_server),
                "ipapi": bool_or_null(ipapi_dc),
                "ipregistry": bool_or_null(reg_server),
                "IPQS": Value::Null,
                "SCAMALYTICS": bool_or_null(scam_is_dc),
                "ipdata": bool_or_null(ipdata_dc),
                "IPinfo": bool_or_null(iio_hosting),
                "IPWHOIS": Value::Null,
                "DBIP": Value::Null
            },
            "Abuser": {
                "IP2LOCATION": bool_or_null(ip2l_abuser),
                "ipapi": bool_or_null(ipapi_abuser),
                "ipregistry": bool_or_null(reg_abuser),
                "IPQS": ipqs_abuser,
                "SCAMALYTICS": bool_or_null(scam_is_blacklisted),
                "ipdata": ipdata_abuser,
                "IPinfo": Value::Null,
                "IPWHOIS": Value::Null,
                "DBIP": dbip_abuser
            },
            "Robot": {
                "IP2LOCATION": bool_or_null(ip2l_robot),
                "ipapi": bool_or_null(ipapi_crawler),
                "IPQS": ipqs_robot,
                "SCAMALYTICS": scam_robot,
                "ipdata": Value::Null,
                "IPinfo": Value::Null,
                "IPWHOIS": Value::Null,
                "DBIP": dbip_robot
            }
        },
        "Media": {
            "TikTok": { "Status": tt_status, "Region": tt_region, "Type": tt_type },
            "DisneyPlus": { "Status": dp_status, "Region": dp_region, "Type": dp_type },
            "Netflix": { "Status": nf_status, "Region": nf_region, "Type": nf_type },
            "Youtube": { "Status": yt_status, "Region": yt_region, "Type": yt_type },
            "AmazonPrimeVideo": { "Status": am_status, "Region": am_region, "Type": am_type },
            "Reddit": { "Status": rd_status, "Region": rd_region, "Type": rd_type },
            "ChatGPT": { "Status": gp_status, "Region": gp_region, "Type": gp_type }
        },
        "Mail": {
            "Port25": port25,
            "Gmail": gmail,
            "Outlook": outlook,
            "Yahoo": yahoo,
            "Apple": apple,
            "QQ": qq,
            "MailRU": mailru,
            "AOL": aol,
            "GMX": gmx,
            "MailCOM": mailcom,
            "163": mail163,
            "Sohu": sohu,
            "Sina": sina,
            "DNSBlacklist": dnsbl
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
        .invoke_handler(tauri::generate_handler![
            run_ip_check,
            run_ip_check_with_args
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
