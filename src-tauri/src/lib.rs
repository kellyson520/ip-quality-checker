use std::fs;
use std::process::Command;

/// Embedded IP quality check script (compiled into the binary)
const IP_SCRIPT: &str = include_str!("../scripts/ip.sh");

/// Run IP quality check in JSON mode (-j flag)
#[tauri::command]
async fn run_ip_check() -> Result<String, String> {
    // Write the embedded script to a temporary file
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("ip_quality_check.sh");

    fs::write(&tmp_path, IP_SCRIPT).map_err(|e| format!("Failed to write temp script: {}", e))?;

    // Ensure it's executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755));
    }

    // Execute: bash <tempfile> -j -n -y
    // -j = JSON output, -n = skip dependency check, -y = auto-install deps
    let output = Command::new("bash")
        .arg(&tmp_path)
        .arg("-j")
        .arg("-n")
        .arg("-y")
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

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
    // Write the embedded script to a temporary file
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("ip_quality_check.sh");

    fs::write(&tmp_path, IP_SCRIPT).map_err(|e| format!("Failed to write temp script: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755));
    }

    // Build command: bash <tempfile> <user args...>
    let mut cmd = Command::new("bash");
    cmd.arg(&tmp_path);
    for arg in &args {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

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
