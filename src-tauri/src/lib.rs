use std::fs;
use std::process::Command;

/// Embedded IP quality check script (compiled into the binary)
const IP_SCRIPT: &str = include_str!("../scripts/ip.sh");

/// Find bash executable path (cross-platform)
fn find_bash() -> Result<String, String> {
    // Unix: bash is always available
    #[cfg(unix)]
    {
        return Ok("bash".to_string());
    }

    // Windows: try common locations
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
            "未找到 bash！请安装 Git for Windows (https://git-scm.com) 并确保添加到 PATH。".to_string()
        );
    }
}

/// Write embedded script to temp file and return path
fn write_temp_script() -> Result<std::path::PathBuf, String> {
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("ip_quality_check.sh");

    fs::write(&tmp_path, IP_SCRIPT)
        .map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Ensure it's executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755));
    }

    Ok(tmp_path)
}

/// Run IP quality check in JSON mode (-j flag)
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    let bash = find_bash()?;
    let tmp_path = write_temp_script()?;

    let output = if bash == "wsl" {
        // Use WSL: wsl bash /tmp/script.sh
        Command::new("wsl")
            .arg("bash")
            .arg(tmp_path.to_string_lossy().to_string())
            .arg("-j")
            .arg("-n")
            .arg("-y")
            .output()
            .map_err(|e| format!("Failed to execute script via WSL: {}", e))?
    } else {
        Command::new(&bash)
            .arg(&tmp_path)
            .arg("-j")
            .arg("-n")
            .arg("-y")
            .output()
            .map_err(|e| format!("Failed to execute script: {}", e))?
    };

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

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
    let tmp_path = write_temp_script()?;

    let output = if bash == "wsl" {
        let mut cmd = Command::new("wsl");
        cmd.arg("bash").arg(tmp_path.to_string_lossy().to_string());
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.output()
            .map_err(|e| format!("Failed to execute script via WSL: {}", e))?
    } else {
        let mut cmd = Command::new(&bash);
        cmd.arg(&tmp_path);
        for arg in &args {
            cmd.arg(arg);
        }
        cmd.output()
            .map_err(|e| format!("Failed to execute script: {}", e))?
    };

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

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
