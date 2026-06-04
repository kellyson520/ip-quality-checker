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

    serde_json::from_str(&text).map_err(|e| {
        format!(
            "JSON parse error: {} | body: {}",
            e,
            &text[..200.min(text.len())]
        )
    })
}

/// Fetch text/HTTP status from URL (mobile only, for non-JSON endpoints)
#[cfg(mobile)]
async fn check_http_status(url: &str) -> u16 {
    match get_client().get(url).send().await {
        Ok(resp) => resp.status().as_u16(),
        Err(_) => 0,
    }
}

/// Fetch raw text from URL (mobile only, for HTML/JSON response parsing)
#[cfg(mobile)]
async fn fetch_text(url: &str) -> Result<String, String> {
    let resp = get_client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .header("Accept-Language", "en")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    resp.text().await.map_err(|e| format!("Read response failed: {}", e))
}

/// Fetch text with custom headers (mobile only)
#[cfg(mobile)]
async fn fetch_text_with_headers(url: &str, headers: &[(&str, &str)]) -> Result<String, String> {
    let mut req = get_client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36");
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let resp = req.send().await.map_err(|e| format!("Request failed: {}", e))?;
    resp.text().await.map_err(|e| format!("Read response failed: {}", e))
}

/// Generate DMS (Degrees, Minutes, Seconds) from latitude and longitude
#[cfg(mobile)]
fn generate_dms(lat: f64, lon: f64) -> String {
    fn convert_single(coord: f64, direction_positive: char, direction_negative: char) -> String {
        let dir = if coord >= 0.0 { direction_positive } else { direction_negative };
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
async fn detect_tiktok(ip: &str) -> (String, Value, Value) {
    // Try main page first
    let body = match fetch_text("https://www.tiktok.com/").await {
        Ok(b) => b,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    // Check for region in response
    if let Some(region) = extract_json_string_field(&body, "region") {
        return ("解锁".into(), Value::String(format!("[{}]", region)), Value::String("Native".into()));
    }
    
    // Try explore page with different headers
    let body2 = match fetch_text_with_headers("https://www.tiktok.com/explore", &[
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
        ("Accept-Encoding", "gzip"),
        ("Accept-Language", "en"),
    ]).await {
        Ok(b) => b,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    if let Some(region) = extract_json_string_field(&body2, "region") {
        return ("IDC".into(), Value::String(format!("[{}]", region)), Value::String("Native".into()));
    }
    
    ("Block".into(), Value::Null, Value::Null)
}

/// Netflix region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_netflix() -> (String, Value, Value) {
    // Check two title URLs
    let url1 = "https://www.netflix.com/title/81280792";
    let url2 = "https://www.netflix.com/title/70143836";
    
    let (r1, r2) = tokio::join!(fetch_text(url1), fetch_text(url2));
    let body1 = r1.unwrap_or_default();
    let body2 = r2.unwrap_or_default();
    
    if body1.is_empty() && body2.is_empty() {
        return ("Block".into(), Value::Null, Value::Null);
    }
    
    // Extract region from JSON
    let region1 = extract_netflix_region(&body1);
    let region2 = extract_netflix_region(&body2);
    let region = if region1.is_some() { region1 } else { region2 };
    
    let has_error1 = body1.contains("Oh no!");
    let has_error2 = body2.contains("Oh no!");
    
    if has_error1 && has_error2 {
        // Only original content available
        let status = if region.is_some() { "仅自制" } else { "Block" };
        (status.into(), region.map(|r| Value::String(format!("[{}]", r))).unwrap_or(Value::Null), Value::Null)
    } else if !has_error1 || !has_error2 {
        // Full unlock
        ("解锁".into(), region.map(|r| Value::String(format!("[{}]", r))).unwrap_or(Value::Null), Value::String("Native".into()))
    } else {
        ("Block".into(), Value::Null, Value::Null)
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
async fn detect_youtube() -> (String, Value, Value) {
    let url = "https://www.youtube.com/premium";
    let body = match fetch_text_with_headers(url, &[
        ("Accept-Language", "en"),
        ("Cookie", "YSC=BiCUU3-5Gdk; CONSENT=YES+cb.20220301-11-p0.en+FX+700; GPS=1; VISITOR_INFO1_LIVE=4VwPMkB7W5A; PREF=tz=Asia.Shanghai"),
    ]).await {
        Ok(b) => b,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    if body.contains("www.google.cn") {
        return ("中国".into(), Value::String("[CN]".into()), Value::Null);
    }
    
    if body.contains("Premium is not available in your country") {
        return ("禁会员".into(), Value::Null, Value::Null);
    }
    
    // Extract region
    let region = extract_youtube_region(&body);
    if body.contains("ad-free") {
        return ("解锁".into(), region.map(|r| Value::String(format!("[{}]", r))).unwrap_or(Value::Null), Value::String("Native".into()));
    }
    
    ("Block".into(), Value::Null, Value::Null)
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
async fn detect_disney() -> (String, Value, Value) {
    let url = "https://disneyplus.com";
    let body = match fetch_text(url).await {
        Ok(b) => b,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    if body.contains("unavailable") || body.contains("not-available") {
        return ("Block".into(), Value::Null, Value::Null);
    }
    
    // Try to extract region from preview page
    if body.contains("preview") {
        return ("Block".into(), Value::Null, Value::Null);
    }
    
    // Simple check - if page loads, consider it accessible
    if body.len() > 1000 {
        ("解锁".into(), Value::Null, Value::String("Native".into()))
    } else {
        ("Block".into(), Value::Null, Value::Null)
    }
}

/// Amazon Prime Video region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_amazon() -> (String, Value, Value) {
    let url = "https://www.primevideo.com";
    let body = match fetch_text(url).await {
        Ok(b) => b,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    // Extract currentTerritory
    if let Some(territory) = extract_amazon_territory(&body) {
        return ("解锁".into(), Value::String(format!("[{}]", territory)), Value::String("Native".into()));
    }
    
    ("Block".into(), Value::Null, Value::Null)
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
async fn detect_reddit() -> (String, Value, Value) {
    let url = "https://www.reddit.com/";
    let resp = match get_client().get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .send().await {
        Ok(r) => r,
        Err(_) => return ("Block".into(), Value::Null, Value::Null),
    };
    
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    
    match status {
        200 => {
            let region = extract_reddit_region(&body);
            ("解锁".into(), region.map(|r| Value::String(format!("[{}]", r))).unwrap_or(Value::Null), Value::String("Native".into()))
        },
        403 => ("Block".into(), Value::Null, Value::Null),
        _ => ("Block".into(), Value::Null, Value::Null),
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
async fn detect_chatgpt() -> (String, Value, Value) {
    // Check multiple endpoints like ip.sh
    let (r1, r2, r3) = tokio::join!(
        fetch_text("https://api.openai.com/compliance/cookie_requirements"),
        fetch_text("https://ios.chat.openai.com/"),
        fetch_text("https://chat.openai.com/cdn-cgi/trace")
    );
    
    let body1 = r1.unwrap_or_default();
    let body2 = r2.unwrap_or_default();
    let trace = r3.unwrap_or_default();
    
    let has_unsupported = body1.contains("unsupported_country");
    let has_vpn = body2.contains("VPN");
    
    // Extract country code from trace
    let country_code = extract_trace_country(&trace);
    
    if !has_unsupported && !has_vpn && !body1.is_empty() && !body2.is_empty() {
        ("解锁".into(), country_code.map(|c| Value::String(format!("[{}]", c))).unwrap_or(Value::Null), Value::String("Native".into()))
    } else if has_vpn && has_unsupported {
        ("Block".into(), Value::Null, Value::Null)
    } else if !has_unsupported && has_vpn {
        ("仅网页".into(), country_code.map(|c| Value::String(format!("[{}]", c))).unwrap_or(Value::Null), Value::String("Native".into()))
    } else if has_unsupported && !has_vpn {
        ("仅APP".into(), country_code.map(|c| Value::String(format!("[{}]", c))).unwrap_or(Value::Null), Value::String("Native".into()))
    } else {
        ("Block".into(), Value::Null, Value::Null)
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
async fn check_bilibili() -> (String, Value, Value) {
    let url = "https://www.bilibili.tv/";
    let status = check_http_status(url).await;
    if status == 200 {
        ("解锁".into(), Value::Null, Value::String("Native".into()))
    } else {
        ("Block".into(), Value::Null, Value::Null)
    }
}

/// Check raw TCP connectivity with a 5-second timeout (mobile only)
#[cfg(mobile)]
async fn check_tcp_connect(addr: &str) -> bool {
    matches!(
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::net::TcpStream::connect(addr),
        )
        .await,
        Ok(Ok(_))
    )
}

/// Check SMTP port 25 connectivity (matching ip.sh's nc-based check)
#[cfg(mobile)]
async fn check_smtp_port25(ip: &str) -> Value {
    // Check if port 25 is reachable via SMTP handshake
    let addrs = ["smtp.gmail.com:25", "smtp.mailgun.org:25"];
    for addr in addrs {
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::net::TcpStream::connect(addr),
        ).await {
            Ok(Ok(stream)) => {
                // Try to read 220 banner
                let readable = stream.readable().await;
                if readable.is_ok() {
                    return Value::String("可用".into());
                }
            },
            _ => continue,
        }
    }
    Value::String("阻断".into())
}

/// Parse DBIP HTML to extract robot/proxy/abuser/risk (matching ip.sh's awk logic)
#[cfg(mobile)]
fn parse_dbip_html(body: &str) -> (Value, Value, Value, Value, Value) {
    if body.is_empty() {
        return (Value::Null, Value::Null, Value::Null, Value::Null, Value::Null);
    }
    
    // Find crawler/proxy/abuser status from HTML
    let mut robot = Value::Null;
    let mut proxy = Value::Null;
    let mut abuser = Value::Null;
    let mut risk_text = Value::Null;
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
    
    // Extract risk level
    if body.contains("low risk") || body.contains("Low") {
        risk_text = Value::String("低风险".into());
    } else if body.contains("medium risk") || body.contains("Medium") {
        risk_text = Value::String("中风险".into());
    } else if body.contains("high risk") || body.contains("High") {
        risk_text = Value::String("高风险".into());
    }
    
    // Simple heuristic: check for proxy/VPN mentions
    let body_lower = body.to_lowercase();
    if body_lower.contains("proxy") || body_lower.contains("vpn") {
        proxy = Value::Bool(true);
    }
    if body_lower.contains("crawler") || body_lower.contains("bot") {
        robot = Value::Bool(true);
    }
    if body_lower.contains("abuse") || body_lower.contains("attack") {
        abuser = Value::Bool(true);
    }
    
    (robot, proxy, abuser, risk_text, country_code)
}

/// Parse IPQS response (matching ip.sh logic)
#[cfg(mobile)]
fn parse_ipqs(data: &Value) -> (f64, Value, Value, Value, Value) {
    let score = data["fraud_score"].as_f64().unwrap_or(0.0);
    let country = data["country_code"].as_str().map(|s| Value::String(s.to_string())).unwrap_or(Value::Null);
    let proxy = Value::Bool(data["proxy"].as_bool().unwrap_or(false));
    let tor = Value::Bool(data["tor"].as_bool().unwrap_or(false));
    let vpn = Value::Bool(data["vpn"].as_bool().unwrap_or(false));
    (score, country, proxy, tor, vpn)
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
    let info_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    let scam_url = format!("https://ipinfo.check.place/{}?db=scamalytics", ip);
    let abuse_url = format!("https://ipinfo.check.place/{}?db=abuseipdb", ip);
    let reg_url = format!("https://ipinfo.check.place/{}?db=ipregistry", ip);
    let ipapi_url = format!("https://ipinfo.check.place/{}?db=ipapi", ip);
    let ip2l_url = format!("https://ipinfo.check.place/{}?db=ip2location", ip);
    let ipdata_url = format!("https://ipinfo.check.place/{}?db=ipdata", ip);
    let ipqs_url = format!("https://ipinfo.check.place/{}?db=ipqualityscore", ip);
    let ipinfo_url = format!("https://ipinfo.io/widget/demo/{}", ip);

    let (info_r, scam_r, abuse_r, reg_r, ipapi_r, ip2l_r, ipdata_r, ipqs_r, ipinfo_r) = tokio::join!(
        fetch_json(&info_url),
        fetch_json(&scam_url),
        fetch_json(&abuse_url),
        fetch_json(&reg_url),
        fetch_json(&ipapi_url),
        fetch_json(&ip2l_url),
        fetch_json(&ipdata_url),
        fetch_json(&ipqs_url),
        fetch_json(&ipinfo_url)
    );

    let info = info_r.unwrap_or(serde_json::json!({}));
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
        (bl_status, _, _),
        (nf_status, nf_region, nf_type),
        (dp_status, dp_region, dp_type),
        (yt_status, yt_region, yt_type),
        (am_status, am_region, am_type),
        (rd_status, rd_region, rd_type),
        (gp_status, gp_region, gp_type)
    ) = tokio::join!(
        detect_tiktok(&ip),
        check_bilibili(),
        detect_netflix(),
        detect_disney(),
        detect_youtube(),
        detect_amazon(),
        detect_reddit(),
        detect_chatgpt()
    );

    // Step 6: Check mail services concurrently via TCP connectivity
    let (gmail, outlook, yahoo, apple, qq, mail163, sohu, sina) = tokio::join!(
        check_tcp_connect("smtp.gmail.com:587"),
        check_tcp_connect("smtp.office365.com:587"),
        check_tcp_connect("smtp.mail.yahoo.com:587"),
        check_tcp_connect("smtp.mail.me.com:587"),
        check_tcp_connect("smtp.qq.com:587"),
        check_tcp_connect("smtp.163.com:465"),
        check_tcp_connect("smtp.sohu.com:465"),
        check_tcp_connect("smtp.sina.com:465")
    );

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
    let lat_val = info["City"]["Latitude"].as_f64();
    let lon_val = info["City"]["Longitude"].as_f64();
    let lat = lat_val.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
    let lon = lon_val.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
    let rad = info["City"]["AccuracyRadius"].as_f64().unwrap_or(0.0);
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

    // DMS and Map: calculate from lat/lon (matching ip.sh logic)
    let (dms, map_url) = if let (Some(lat_f), Some(lon_f)) = (lat_val, lon_val) {
        let dms_str = generate_dms(lat_f, lon_f);
        let map_str = generate_map_url(lat_f, lon_f, rad);
        (Value::String(dms_str), Value::String(map_str))
    } else {
        (Value::Null, Value::Null)
    };

    // Info.Type: compare country vs registered country
    let info_type = if country_code != "null"
        && !country_code.is_empty()
        && reg_country_code != "null"
        && !reg_country_code.is_empty()
    {
        if country_code == reg_country_code {
            Value::String("本土IP地址".to_string())
        } else {
            Value::String("海外IP地址".to_string())
        }
    } else {
        Value::Null
    };

    // Scamalytics
    let scam_score = jf64(&scam, &["scamalytics", "scamalytics_score"]);
    let scam_is_vpn = jbool(&scam, &["scamalytics", "scamalytics_proxy", "is_vpn"]);
    let scam_is_dc = jbool(
        &scam,
        &["scamalytics", "scamalytics_proxy", "is_datacenter"],
    );
    let scam_is_tor = jbool(&scam, &["external_datasources", "x4bnet", "is_tor"]);
    let scam_is_proxy = jbool(&scam, &["external_datasources", "firehol", "is_proxy"]);
    let scam_is_blacklisted = jbool(&scam, &["scamalytics", "is_blacklisted_external"]);
    let scam_country = jstr(
        &scam,
        &[
            "external_datasources",
            "maxmind_geolite2",
            "ip_country_code",
        ],
    );
    let scam_robot = jbool(
        &scam,
        &["external_datasources", "x4bnet", "is_blacklisted_spambot"],
    ) || jbool(
        &scam,
        &["external_datasources", "x4bnet", "is_bot_operamini"],
    ) || jbool(&scam, &["external_datasources", "x4bnet", "is_bot_semrush"]);

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
    let ipapi_score = jf64(&ipapi, &["fraud_score"]);
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
    let ip2l_score = jf64(&ip2l, &["fraud_score"]);

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

    // IPQS (ipqualityscore)
    let (ipqs_score, ipqs_country, ipqs_proxy, ipqs_tor, ipqs_vpn) = parse_ipqs(&ipqs);
    let ipqs_abuser = ipqs["recent_abuse"].as_bool().unwrap_or(false);
    let ipqs_robot = ipqs["bot_status"].as_bool().unwrap_or(false);

    // DBIP (parse HTML)
    let (dbip_robot, dbip_proxy, dbip_abuser, dbip_risk, dbip_country) = parse_dbip_html(&dbip_body);

    // Port 25 check
    let port25 = check_smtp_port25(&ip).await;

    // === Build unified output ===

    // Type.Usage: collect from all sources
    let mut usage_map = serde_json::Map::new();
    if iio_usage != "null" && !iio_usage.is_empty() {
        usage_map.insert("IPinfo".into(), Value::String(iio_usage));
    }
    if reg_usage != "null" && !reg_usage.is_empty() {
        usage_map.insert("ipregistry".into(), Value::String(reg_usage));
    }
    if ipapi_usage != "null" && !ipapi_usage.is_empty() {
        usage_map.insert("ipapi".into(), Value::String(ipapi_usage));
    }
    if abuse_usage != "null" && !abuse_usage.is_empty() {
        usage_map.insert("AbuseIPDB".into(), Value::String(abuse_usage));
    }
    if ip2l_usage != "null" && !ip2l_usage.is_empty() {
        usage_map.insert("IP2LOCATION".into(), Value::String(ip2l_usage));
    }

    // Type.Company: collect from all sources
    let mut company_map = serde_json::Map::new();
    if iio_company_type != "null" && !iio_company_type.is_empty() {
        company_map.insert("IPinfo".into(), Value::String(iio_company_type));
    }
    if reg_company_type != "null" && !reg_company_type.is_empty() {
        company_map.insert("ipregistry".into(), Value::String(reg_company_type));
    }
    if ipapi_company_type != "null" && !ipapi_company_type.is_empty() {
        company_map.insert("ipapi".into(), Value::String(ipapi_company_type));
    }

    // Score: weighted average of all available sources (matching ip.sh)
    let total_score = {
        let mut total = 0.0;
        let mut count = 0u32;
        if scam_score > 0.0 { total += scam_score; count += 1; }
        if abuse_score > 0.0 { total += abuse_score; count += 1; }
        if ipapi_score > 0.0 { total += ipapi_score; count += 1; }
        if ip2l_score > 0.0 { total += ip2l_score; count += 1; }
        if ipqs_score > 0.0 { total += ipqs_score; count += 1; }
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
            "DMS": dms,
            "Map": map_url,
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
            "IP2LOCATION": format!("{}", ip2l_score as u32),
            "SCAMALYTICS": format!("{}", scam_score as u32),
            "ipapi": format!("{}", ipapi_score as u32),
            "AbuseIPDB": format!("{}", abuse_score as u32),
            "IPQS": format!("{}", ipqs_score as u32),
            "DBIP": dbip_risk
        },
        "Factor": {
            "CountryCode": {
                "IP2LOCATION": ip2l_country != "null" && !ip2l_country.is_empty(),
                "ipapi": ipapi_country != "null" && !ipapi_country.is_empty(),
                "ipregistry": reg_country != "null" && !reg_country.is_empty(),
                "IPQS": ipqs_country != Value::Null,
                "SCAMALYTICS": scam_country != "null" && !scam_country.is_empty(),
                "ipdata": ipdata_country != "null" && !ipdata_country.is_empty(),
                "IPinfo": iio_country != "null" && !iio_country.is_empty(),
                "IPWHOIS": false,
                "DBIP": dbip_country != Value::Null
            },
            "Proxy": {
                "scamalytics": scam_is_proxy,
                "ipregistry": reg_proxy,
                "ipapi": ipapi_proxy,
                "ipdata": ipdata_proxy,
                "IPinfo": iio_proxy,
                "IPQS": ipqs_proxy.as_bool().unwrap_or(false),
                "DBIP": dbip_proxy.as_bool().unwrap_or(false)
            },
            "Tor": {
                "scamalytics": scam_is_tor,
                "ipregistry": reg_tor,
                "ipapi": ipapi_tor,
                "AbuseIPDB": abuse_is_tor,
                "ipdata": ipdata_tor,
                "IPinfo": iio_tor,
                "IPQS": ipqs_tor.as_bool().unwrap_or(false)
            },
            "VPN": {
                "scamalytics": scam_is_vpn,
                "ipregistry": reg_vpn,
                "ipapi": ipapi_vpn,
                "IPinfo": iio_vpn,
                "IPQS": ipqs_vpn.as_bool().unwrap_or(false)
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
                "ipdata": ipdata_abuser,
                "IPQS": ipqs_abuser,
                "DBIP": dbip_abuser.as_bool().unwrap_or(false)
            },
            "Robot": {
                "scamalytics": scam_robot,
                "ipapi": ipapi_crawler,
                "IPQS": ipqs_robot,
                "DBIP": dbip_robot.as_bool().unwrap_or(false)
            }
        },
        "Media": {
            "TikTok": { "Status": tt_status, "Region": tt_region, "Type": tt_type },
            "Bilibili": { "Status": bl_status, "Region": Value::Null, "Type": Value::Null },
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
            "163": mail163,
            "Sohu": sohu,
            "Sina": sina,
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
        .invoke_handler(tauri::generate_handler![
            run_ip_check,
            run_ip_check_with_args
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
