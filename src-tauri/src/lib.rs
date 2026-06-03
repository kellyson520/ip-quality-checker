use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// Embedded IP quality check script (compiled into the binary, desktop only)
#[cfg(desktop)]
const IP_SCRIPT: &str = include_str!("../scripts/ip.sh");

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
#[cfg(desktop)]
#[tauri::command]
async fn run_ip_check_with_args(args: Vec<String>) -> Result<String, String> {
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

/// Native IP check using HTTP APIs (mobile - Android/iOS)
#[cfg(mobile)]
async fn fetch_json(url: &str) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Read response failed: {}", e))?;

    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {} | body: {}", e, &text[..200.min(text.len())]))
}

/// Native IP check using HTTP APIs (mobile)
#[cfg(mobile)]
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    // Step 1: Get public IP
    let ip_resp = fetch_json("https://api.ipify.org?format=json").await?;
    let ip = ip_resp["ip"]
        .as_str()
        .ok_or("Failed to get IP from response")?
        .to_string();

    // Step 2: Get IP info from ipinfo.check.place
    let info_url = format!("https://ipinfo.check.place/{}?lang=zh-CN", ip);
    let info = fetch_json(&info_url).await.unwrap_or(serde_json::json!({}));

    // Step 3: Get scamalytics data
    let scam_url = format!("https://ipinfo.check.place/{}?db=scamalytics", ip);
    let scam = fetch_json(&scam_url).await.unwrap_or(serde_json::json!({}));

    // Step 4: Get abuseipdb data
    let abuse_url = format!("https://ipinfo.check.place/{}?db=abuseipdb", ip);
    let abuse = fetch_json(&abuse_url).await.unwrap_or(serde_json::json!({}));

    // Step 5: Get ipqualityscore data
    let ipqs_url = format!("https://ipinfo.check.place/{}?db=ipqualityscore", ip);
    let ipqs = fetch_json(&ipqs_url).await.unwrap_or(serde_json::json!({}));

    // Step 6: Check streaming services
    let mut media = serde_json::json!({});

    // Netflix
    if let Ok(nf) = fetch_json("https://netflix.com/title/81280792").await {
        media["Netflix"] = nf;
    }

    // ChatGPT
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let chatgpt_status = match client.get("https://chatgpt.com/").send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if status == 200 { "Y" } else { "N" }
        }
        Err(_) => "N",
    };
    media["ChatGPT"] = serde_json::json!({"status": chatgpt_status});

    // Build result matching ip.sh JSON format
    let result = serde_json::json!({
        "Head": {
            "IP": ip,
            "Time": chrono_now(),
            "Version": "mobile-native"
        },
        "Info": info,
        "Type": {
            "Proxy": scam.get("proxy").unwrap_or(&serde_json::json!("")).to_string().trim_matches('"') != "no",
            "VPN": scam.get("vpn").unwrap_or(&serde_json::json!("")).to_string().trim_matches('"') != "no",
            "Tor": scam.get("tor").unwrap_or(&serde_json::json!("")).to_string().trim_matches('"') != "no"
        },
        "Score": {
            "Total": calculate_score(&scam, &abuse, &ipqs),
            "Scamalytics": scam.get("score").unwrap_or(&serde_json::json!(0)).clone(),
            "AbuseIPDB": abuse.get("abuseConfidenceScore").unwrap_or(&serde_json::json!(0)).clone(),
            "IPQS": ipqs.get("fraud_score").unwrap_or(&serde_json::json!(0)).clone()
        },
        "Factor": {
            "Proxy": scam.get("proxy").unwrap_or(&serde_json::json!("unknown")).clone(),
            "VPN": scam.get("vpn").unwrap_or(&serde_json::json!("unknown")).clone(),
            "Tor": scam.get("tor").unwrap_or(&serde_json::json!("unknown")).clone(),
            "Abuse": abuse.get("abuseConfidenceScore").unwrap_or(&serde_json::json!(0)).clone()
        },
        "Media": media,
        "Mail": {}
    });

    serde_json::to_string(&result).map_err(|e| format!("Serialize error: {}", e))
}

#[cfg(mobile)]
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(mobile)]
fn calculate_score(scam: &Value, abuse: &Value, ipqs: &Value) -> u32 {
    let scam_score = scam.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
    let abuse_score = abuse.get("abuseConfidenceScore").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
    let ipqs_score = ipqs.get("fraud_score").and_then(|v| v.as_f64()).unwrap_or(0.0) as u32;
    // Weighted average
    (scam_score + abuse_score + ipqs_score) / 3
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
