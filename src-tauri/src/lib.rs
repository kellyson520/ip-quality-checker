use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// Embedded IP quality check script (compiled into the binary)
const IP_SCRIPT: &str = include_str!("../scripts/ip.sh");

/// Find bash executable path (cross-platform)
fn find_bash() -> Result<String, String> {
    #[cfg(unix)]
    {
        return Ok("bash".to_string());
    }

    #[cfg(windows)]
    {
        // 1. Try PATH first
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

        // 2. Try Git for Windows bash
        let git_bash_paths = vec![
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for path in git_bash_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // 3. Try WSL
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

/// Execute script via stdin pipe (avoids noexec temp directory issues)
fn exec_via_stdin(bash: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let mut cmd = if bash == "wsl" {
        let mut c = Command::new("wsl");
        c.arg("bash").arg("-s");
        c
    } else {
        let mut c = Command::new(bash);
        c.arg("-s"); // read script from stdin
        c
    };

    // Pass arguments after "--" separator
    for arg in args {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start script: {}", e))?;

    // Write script content to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(IP_SCRIPT.as_bytes())
            .map_err(|e| format!("Failed to write script to stdin: {}", e))?;
        // stdin is dropped here, closing the pipe
    }

    child
        .wait_with_output()
        .map_err(|e| format!("Failed to execute script: {}", e))
}

/// Run IP quality check in JSON mode (-j flag)
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

/// Run IP quality check with custom arguments
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

/// Build and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_ip_check, run_ip_check_with_args])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
