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
    // Convert to UTC datetime components
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    // Days since epoch to Y-M-D (simplified leap year calculation)
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

/// Calculate weighted score from multiple sources (mobile only)
/// Only includes sources with non-zero scores in the average
#[cfg(mobile)]
fn calculate_score(scam: &Value, abuse: &Value, ipqs: &Value) -> u32 {
    let scam_score = scam.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let abuse_score = abuse
        .get("abuseConfidenceScore")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let ipqs_score = ipqs.get("fraud_score").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let mut total = 0.0;
    let mut count = 0u32;
    if scam_score > 0.0 {
        total += scam_score;
        count += 1;
    }
    if abuse_score > 0.0 {
        total += abuse_score;
        count += 1;
    }
    if ipqs_score > 0.0 {
        total += ipqs_score;
        count += 1;
    }
    if count > 0 {
        (total / count as f64) as u32
    } else {
        0
    }
}

/// Native IP check using HTTP APIs (mobile - Android/iOS)
/// Output format matches the bash script (ip.sh) JSON output exactly
#[cfg(mobile)]
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    // Step 1: Get public IP
    let ip_resp = fetch_json("https://api.ipify.org?format=json").await?;
    let ip = ip_resp["ip"]
        .as_str()
        .ok_or("Failed to get IP from response")?
        .to_string();

    // Step 2-5: Concurrent API requests
    let info_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    let scam_url = format!("https://ipinfo.check.place/{}?db=scamalytics", ip);
    let abuse_url = format!("https://ipinfo.check.place/{}?db=abuseipdb", ip);
    let ipqs_url = format!("https://ipinfo.check.place/{}?db=ipqualityscore", ip);

    let (info_r, scam_r, abuse_r, ipqs_r) = tokio::join!(
        fetch_json(&info_url),
        fetch_json(&scam_url),
        fetch_json(&abuse_url),
        fetch_json(&ipqs_url)
    );

    let info = info_r.unwrap_or(serde_json::json!({}));
    let scam = scam_r.unwrap_or(serde_json::json!({}));
    let abuse = abuse_r.unwrap_or(serde_json::json!({}));
    let ipqs = ipqs_r.unwrap_or(serde_json::json!({}));

    // Step 6: Check streaming services concurrently
    let (nf, dp, yt, am, rd, gp) = tokio::join!(
        check_http_status("https://www.netflix.com/title/81280792"),
        check_http_status("https://www.disneyplus.com/"),
        check_http_status("https://www.youtube.com/"),
        check_http_status("https://www.amazon.com/gp/video/storefront"),
        check_http_status("https://www.reddit.com/"),
        check_http_status("https://chatgpt.com/")
    );

    let status_to_yn = |code: u16| if code == 200 { "Y" } else { "N" };

    // Build result matching ip.sh JSON structure exactly
    let result = serde_json::json!({
        "Head": {
            "IP": ip,
            "Time": chrono_now(),
            "Version": "mobile-native"
        },
        "Info": info,
        "Type": {
            "Usage": {
                "usage_type": scam.get("usage_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string()
            },
            "Company": {
                "company_type": scam.get("company_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string()
            }
        },
        "Score": {
            "Total": calculate_score(&scam, &abuse, &ipqs).to_string(),
            "Scamalytics": scam.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
            "AbuseIPDB": abuse.get("abuseConfidenceScore").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
            "IPQS": ipqs.get("fraud_score").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string()
        },
        "Factor": {
            "CountryCode": {
                scam.get("country_code").and_then(|v| v.as_str()).unwrap_or("XX").to_string(): true
            },
            "Proxy": {
                "scamalytics": scam.get("proxy").and_then(|v| v.as_str()).unwrap_or("unknown") != "no",
                "ipqs": ipqs.get("proxy").and_then(|v| v.as_bool()).unwrap_or(false)
            },
            "Tor": {
                "scamalytics": scam.get("tor").and_then(|v| v.as_str()).unwrap_or("unknown") != "no",
                "ipqs": ipqs.get("tor").and_then(|v| v.as_bool()).unwrap_or(false)
            },
            "VPN": {
                "scamalytics": scam.get("vpn").and_then(|v| v.as_str()).unwrap_or("unknown") != "no",
                "ipqs": ipqs.get("vpn").and_then(|v| v.as_bool()).unwrap_or(false)
            },
            "Abuser": {
                "abuseipdb": abuse.get("abuseConfidenceScore").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0,
                "ipqs": ipqs.get("abuse_score").and_then(|v| v.as_f64()).unwrap_or(0.0) > 0.0
            }
        },
        "Media": {
            "TikTok": { "Result": "N" },
            "DisneyPlus": { "Result": status_to_yn(dp) },
            "Netflix": { "Result": status_to_yn(nf) },
            "YouTube": { "Result": status_to_yn(yt) },
            "AmazonPrime": { "Result": status_to_yn(am) },
            "Reddit": { "Result": status_to_yn(rd) },
            "ChatGPT": { "Result": status_to_yn(gp) }
        },
        "Mail": {
            "Port25": { "Status": "unknown", "Port": "25" },
            "ServiceName": { "Status": "unknown", "Port": "53" },
            "DNSBlacklist": { "Status": "unknown", "Port": "25" }
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
