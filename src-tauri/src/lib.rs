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
    jvalue(v, path).and_then(|cur| {
        cur.as_f64().or_else(|| cur.as_str()?.parse::<f64>().ok())
    })
}

/// Helper: get bool from nested JSON path.
#[cfg(mobile)]
fn opt_bool(v: &Value, path: &[&str]) -> Option<bool> {
    jvalue(v, path).and_then(|cur| {
        cur.as_bool().or_else(|| match cur.as_str()?.to_ascii_lowercase().as_str() {
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

/// Fetch JSON from URL using shared client (mobile only)
#[cfg(mobile)]
async fn fetch_json(url: &str) -> Result<Value, String> {
    let resp = get_client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
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
    let primary_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    if let Ok(data) = fetch_json(&primary_url).await {
        if !data.is_null() && data.is_object() {
            return data;
        }
    }

    let fallback_url = format!("https://ipinfo.check.place/{}?lang=en", ip);
    if let Ok(data) = fetch_json(&fallback_url).await {
        if !data.is_null() && data.is_object() {
            return data;
        }
    }

    Value::Null
}

/// Fetch text/HTTP status from URL (mobile only, for non-JSON endpoints)
#[cfg(mobile)]
async fn check_http_status(url: &str) -> u16 {
    match get_client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .send()
        .await {
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

#[cfg(mobile)]
async fn fetch_ipregistry(ip: &str) -> Result<Value, String> {
    let html = fetch_text("https://ipregistry.co").await.unwrap_or_default();
    let key = html
        .split("apiKey=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .filter(|value| !value.is_empty())
        .unwrap_or("sb69ksjcajfs4c");
    let url = format!("https://api.ipregistry.co/{}?hostname=true&key={}", ip, key);
    fetch_json(&url).await
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
async fn detect_tiktok(_ip: &str) -> (Value, Value, Value) {
    // Try main page first
    let body = match fetch_text("https://www.tiktok.com/").await {
        Ok(b) => b,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    
    // Check for region in response
    if let Some(region) = extract_json_string_field(&body, "region") {
        return (
            Value::String("Yes".into()),
            region_or_null(Some(region)),
            Value::String("Native".into()),
        );
    }
    
    // Try explore page with different headers
    let body2 = match fetch_text_with_headers("https://www.tiktok.com/explore", &[
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"),
        ("Accept-Language", "en"),
    ]).await {
        Ok(b) => b,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    
    if let Some(region) = extract_json_string_field(&body2, "region") {
        return (
            Value::String("IDC".into()),
            region_or_null(Some(region)),
            Value::String("Native".into()),
        );
    }
    
    (Value::String("Block".into()), Value::Null, Value::Null)
}

/// Netflix region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_netflix() -> (Value, Value, Value) {
    // Check two title URLs
    let url1 = "https://www.netflix.com/title/81280792";
    let url2 = "https://www.netflix.com/title/70143836";
    
    let (r1, r2) = tokio::join!(fetch_text(url1), fetch_text(url2));
    let body1 = r1.unwrap_or_default();
    let body2 = r2.unwrap_or_default();
    
    if body1.is_empty() || body2.is_empty() {
        return (Value::Null, Value::Null, Value::Null);
    }
    
    // Extract region from JSON
    let region1 = extract_netflix_region(&body1);
    let region2 = extract_netflix_region(&body2);
    let region = if region1.is_some() { region1 } else { region2 };
    
    let has_error1 = body1.contains("Oh no!");
    let has_error2 = body2.contains("Oh no!");
    
    if has_error1 && has_error2 {
        // Only original content available
        let status = if region.is_some() { "NF.Only" } else { "Block" };
        (Value::String(status.into()), region_or_null(region), Value::Null)
    } else if !has_error1 || !has_error2 {
        // Full unlock
        (
            Value::String("Yes".into()),
            region_or_null(region),
            Value::String("Native".into()),
        )
    } else {
        (Value::String("Block".into()), Value::Null, Value::Null)
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
    let url = "https://www.youtube.com/premium";
    let body = match fetch_text_with_headers(url, &[
        ("Accept-Language", "en"),
        ("Cookie", "YSC=BiCUU3-5Gdk; CONSENT=YES+cb.20220301-11-p0.en+FX+700; GPS=1; VISITOR_INFO1_LIVE=4VwPMkB7W5A; PREF=tz=Asia.Shanghai"),
    ]).await {
        Ok(b) => b,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    
    if body.contains("www.google.cn") {
        return (Value::String("China".into()), Value::String("CN".into()), Value::Null);
    }
    
    if body.contains("Premium is not available in your country") {
        return (Value::String("NoPrem.".into()), Value::Null, Value::Null);
    }
    
    // Extract region
    let region = extract_youtube_region(&body);
    if body.contains("ad-free") {
        return (
            Value::String("Yes".into()),
            region_or_null(region),
            Value::String("Native".into()),
        );
    }
    
    (Value::String("Failed".into()), Value::Null, Value::Null)
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
    const AUTH: &str = "Bearer ZGlzbmV5JmJyb3dzZXImMS4wLjA.Cu56AgSfBTDag5NiRA81oLHkDZfu5L3CKadnefEAY84";
    let assertion_resp = match get_client()
        .post("https://disney.api.edge.bamgrid.com/devices")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .header("authorization", AUTH)
        .header("content-type", "application/json; charset=UTF-8")
        .body(r#"{"deviceFamily":"browser","applicationRuntime":"chrome","deviceProfile":"windows","attributes":{}}"#)
        .send()
        .await
    {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };

    let assertion_json: Value = match serde_json::from_str(&assertion_resp) {
        Ok(v) => v,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    let assertion = match opt_str(&assertion_json, &["assertion"]) {
        Some(v) => v,
        None => return (Value::Null, Value::Null, Value::Null),
    };

    let token_body = format!(
        "grant_type=urn:ietf:params:oauth:grant-type:token-exchange&platform=browser&subject_token={}&subject_token_type=urn:bamtech:params:oauth:token-type:device",
        assertion
    );
    let token_resp = match get_client()
        .post("https://disney.api.edge.bamgrid.com/token")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .header("authorization", AUTH)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(token_body)
        .send()
        .await
    {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    if token_resp.contains("forbidden-location") || token_resp.contains("403 ERROR") {
        return (Value::String("Block".into()), Value::Null, Value::Null);
    }
    let token_json: Value = match serde_json::from_str(&token_resp) {
        Ok(v) => v,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    let refresh_token = match opt_str(&token_json, &["refresh_token"]) {
        Some(v) => v,
        None => return (Value::Null, Value::Null, Value::Null),
    };

    let graphql_body = serde_json::json!({
        "query": "query { sdk { session { inSupportedLocation location { countryCode } } } }",
        "variables": {},
        "operationName": null,
        "extensions": {
            "sdk": {
                "token": {
                    "accessToken": refresh_token
                }
            }
        }
    });
    let gql_resp = match get_client()
        .post("https://disney.api.edge.bamgrid.com/graph/v1/device/graphql")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .header("authorization", "ZGlzbmV5JmJyb3dzZXImMS4wLjA.Cu56AgSfBTDag5NiRA81oLHkDZfu5L3CKadnefEAY84")
        .json(&graphql_body)
        .send()
        .await
    {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    let gql_json: Value = match serde_json::from_str(&gql_resp) {
        Ok(v) => v,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    let region = opt_str(&gql_json, &["extensions", "sdk", "session", "location", "countryCode"]);
    let supported = opt_bool(&gql_json, &["extensions", "sdk", "session", "inSupportedLocation"]);
    match (region, supported) {
        (Some(region), Some(true)) => (
            Value::String("Yes".into()),
            Value::String(region),
            Value::String("Native".into()),
        ),
        (Some(region), Some(false)) => (
            Value::String("Pending".into()),
            Value::String(region),
            Value::String("Native".into()),
        ),
        (None, _) => (Value::String("Block".into()), Value::Null, Value::Null),
        (Some(region), None) => (Value::Null, Value::String(region), Value::Null),
    }
}

/// Amazon Prime Video region detection (matching ip.sh logic)
#[cfg(mobile)]
async fn detect_amazon() -> (Value, Value, Value) {
    let url = "https://www.primevideo.com";
    let body = match fetch_text(url).await {
        Ok(b) => b,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    
    // Extract currentTerritory
    if let Some(territory) = extract_amazon_territory(&body) {
        return (
            Value::String("Yes".into()),
            Value::String(territory),
            Value::String("Native".into()),
        );
    }
    
    (Value::String("Block".into()), Value::Null, Value::Null)
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
    let url = "https://www.reddit.com/";
    let resp = match get_client().get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
        .send().await {
        Ok(r) => r,
        Err(_) => return (Value::Null, Value::Null, Value::Null),
    };
    
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    
    match status {
        200 => {
            let region = extract_reddit_region(&body);
            (
                Value::String("Yes".into()),
                region_or_null(region),
                Value::String("Native".into()),
            )
        },
        403 => (Value::String("Block".into()), Value::Null, Value::Null),
        _ => (Value::String("Failed".into()), Value::Null, Value::Null),
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
        (
            Value::String("Yes".into()),
            region_or_null(country_code),
            Value::String("Native".into()),
        )
    } else if has_vpn && has_unsupported {
        (Value::String("Block".into()), Value::Null, Value::Null)
    } else if !has_unsupported && has_vpn {
        (
            Value::String("WebOnly".into()),
            region_or_null(country_code),
            Value::String("Native".into()),
        )
    } else if has_unsupported && !has_vpn {
        (
            Value::String("APPOnly".into()),
            region_or_null(country_code),
            Value::String("Native".into()),
        )
    } else {
        (Value::Null, Value::Null, Value::Null)
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
        (Value::String("Yes".into()), Value::Null, Value::String("Native".into()))
    } else if status == 0 {
        (Value::Null, Value::Null, Value::Null)
    } else {
        (Value::String("Block".into()), Value::Null, Value::Null)
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
    let read = match tokio::time::timeout(std::time::Duration::from_secs(5), stream.read(&mut buf)).await {
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
    match check_smtp_banner("smtp.mailgun.org:25").await {
        Some(true) => Value::Bool(true),
        Some(false) => Value::Bool(false),
        None => Value::Null,
    }
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

    // Step 6: Check mail services via MX records on SMTP port 25.
    let (gmail, outlook, yahoo, apple, qq, mailru, aol, gmx, mailcom, mail163, sohu, sina) = tokio::join!(
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
    );

    // === Map API responses to bash script JSON format ===

    // Info: from ipinfo.check.place (maxmind data)
    let asn = info["ASN"]["AutonomousSystemNumber"]
        .as_u64()
        .map(|v| Value::String(v.to_string()))
        .unwrap_or(Value::Null);
    let org = string_or_null(opt_str(&info, &["ASN", "AutonomousSystemOrganization"]));
    let city_name = string_or_null(opt_str(&info, &["City", "Name"]));
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
    let reg_country_name = string_or_null(opt_str(&info, &["Country", "RegisteredCountry", "Name"]));
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
    let ip2l_score = opt_f64(&ip2l, &["fraud_score"]);

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
    let (dbip_robot, dbip_proxy, dbip_abuser, dbip_risk, dbip_country) = parse_dbip_html(&dbip_body);

    // Port 25 check
    let port25 = check_smtp_port25(&ip).await;

    // === Build unified output ===

    // Type.Usage: collect from all sources
    let mut usage_map = serde_json::Map::new();
    if let Some(value) = iio_usage {
        usage_map.insert("IPinfo".into(), Value::String(value));
    }
    if let Some(value) = reg_usage {
        usage_map.insert("ipregistry".into(), Value::String(value));
    }
    if let Some(value) = ipapi_usage {
        usage_map.insert("ipapi".into(), Value::String(value));
    }
    if let Some(value) = abuse_usage {
        usage_map.insert("AbuseIPDB".into(), Value::String(value));
    }
    if let Some(value) = ip2l_usage {
        usage_map.insert("IP2LOCATION".into(), Value::String(value));
    }

    // Type.Company: collect from all sources
    let mut company_map = serde_json::Map::new();
    if let Some(value) = iio_company_type {
        company_map.insert("IPinfo".into(), Value::String(value));
    }
    if let Some(value) = reg_company_type {
        company_map.insert("ipregistry".into(), Value::String(value));
    }
    if let Some(value) = ipapi_company_type {
        company_map.insert("ipapi".into(), Value::String(value));
    }

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
            "Region": { "Code": string_or_null(country_code.clone()), "Name": country_name },
            "Continent": { "Code": continent_code, "Name": continent_name },
            "RegisteredRegion": { "Code": string_or_null(reg_country_code.clone()), "Name": reg_country_name },
            "Type": info_type
        },
        "Type": {
            "Usage": Value::Object(usage_map),
            "Company": Value::Object(company_map)
        },
        "Score": {
            "IP2LOCATION": number_string_or_null(ip2l_score),
            "SCAMALYTICS": number_string_or_null(scam_score),
            "ipapi": number_string_or_null(ipapi_score),
            "AbuseIPDB": number_string_or_null(abuse_score),
            "IPQS": number_string_or_null(ipqs_score),
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
                "scamalytics": bool_or_null(scam_is_proxy),
                "ipregistry": bool_or_null(reg_proxy),
                "ipapi": bool_or_null(ipapi_proxy),
                "ipdata": bool_or_null(ipdata_proxy),
                "IPinfo": bool_or_null(iio_proxy),
                "IPQS": ipqs_proxy,
                "DBIP": dbip_proxy
            },
            "Tor": {
                "scamalytics": bool_or_null(scam_is_tor),
                "ipregistry": bool_or_null(reg_tor),
                "ipapi": bool_or_null(ipapi_tor),
                "AbuseIPDB": bool_or_null(abuse_is_tor),
                "ipdata": bool_or_null(ipdata_tor),
                "IPinfo": bool_or_null(iio_tor),
                "IPQS": ipqs_tor
            },
            "VPN": {
                "scamalytics": bool_or_null(scam_is_vpn),
                "ipregistry": bool_or_null(reg_vpn),
                "ipapi": bool_or_null(ipapi_vpn),
                "IPinfo": bool_or_null(iio_vpn),
                "IPQS": ipqs_vpn
            },
            "Server": {
                "scamalytics": bool_or_null(scam_is_dc),
                "ipregistry": bool_or_null(reg_server),
                "ipapi": bool_or_null(ipapi_dc),
                "ipdata": bool_or_null(ipdata_dc),
                "IPinfo": bool_or_null(iio_hosting),
                "IPQS": Value::Null,
                "DBIP": Value::Null
            },
            "Abuser": {
                "scamalytics": bool_or_null(scam_is_blacklisted),
                "ipregistry": bool_or_null(reg_abuser),
                "ipapi": bool_or_null(ipapi_abuser),
                "ipdata": ipdata_abuser,
                "IPQS": ipqs_abuser,
                "DBIP": dbip_abuser
            },
            "Robot": {
                "scamalytics": scam_robot,
                "ipapi": bool_or_null(ipapi_crawler),
                "IPQS": ipqs_robot,
                "DBIP": dbip_robot
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
            "MailRU": mailru,
            "AOL": aol,
            "GMX": gmx,
            "MailCOM": mailcom,
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
