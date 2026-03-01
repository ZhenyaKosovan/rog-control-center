use std::process::Command;

use crate::error::{AppError, Result};

pub fn get_mode() -> Result<String> {
    let output = Command::new("supergfxctl")
        .arg("-g")
        .output()
        .map_err(|e| AppError::Command {
            cmd: "supergfxctl -g".into(),
            msg: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Command {
            cmd: "supergfxctl -g".into(),
            msg: stderr,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn get_supported_modes() -> Result<Vec<String>> {
    let output = Command::new("supergfxctl")
        .arg("-s")
        .output()
        .map_err(|e| AppError::Command {
            cmd: "supergfxctl -s".into(),
            msg: e.to_string(),
        })?;

    let out = String::from_utf8_lossy(&output.stdout).to_string();
    // "[Integrated, Hybrid, AsusMuxDgpu]"
    let out = out.trim().trim_start_matches('[').trim_end_matches(']');
    Ok(out.split(',').map(|s| s.trim().to_string()).collect())
}

pub fn set_mode(mode: &str) -> Result<()> {
    let output = Command::new("supergfxctl")
        .args(["-m", mode])
        .output()
        .map_err(|e| AppError::Command {
            cmd: format!("supergfxctl -m {mode}"),
            msg: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Command {
            cmd: format!("supergfxctl -m {mode}"),
            msg: stderr,
        });
    }

    // Signal waybar after GPU mode change
    let _ = Command::new("pkill")
        .args(["-RTMIN+11", "waybar"])
        .spawn();

    Ok(())
}
