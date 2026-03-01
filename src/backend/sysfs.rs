use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// Discovered hwmon paths keyed by sensor name
pub struct HwmonPaths {
    pub k10temp: Option<PathBuf>,    // CPU temp
    pub amdgpu: Vec<PathBuf>,        // GPU temp(s)
    pub asus_fans: Option<PathBuf>,  // fan1_input, fan2_input
}

impl HwmonPaths {
    pub fn discover() -> Self {
        let mut paths = Self {
            k10temp: None,
            amdgpu: Vec::new(),
            asus_fans: None,
        };

        let hwmon_base = Path::new("/sys/class/hwmon");
        if let Ok(entries) = fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(name) = fs::read_to_string(path.join("name")) {
                    let name = name.trim();
                    match name {
                        "k10temp" => paths.k10temp = Some(path),
                        "amdgpu" => paths.amdgpu.push(path),
                        "asus" => paths.asus_fans = Some(path),
                        _ => {}
                    }
                }
            }
        }

        paths
    }
}

pub fn read_temp(hwmon_path: &Path) -> Result<f64> {
    let val = fs::read_to_string(hwmon_path.join("temp1_input"))
        .map_err(|e| AppError::Io(e))?;
    val.trim()
        .parse::<f64>()
        .map(|v| v / 1000.0)
        .map_err(|e| AppError::Parse(format!("temp parse: {e}")))
}

pub fn read_fan_rpm(hwmon_path: &Path, fan_idx: u8) -> Result<u32> {
    let file = format!("fan{}_input", fan_idx);
    let val = fs::read_to_string(hwmon_path.join(file))
        .map_err(|e| AppError::Io(e))?;
    val.trim()
        .parse()
        .map_err(|e| AppError::Parse(format!("fan RPM parse: {e}")))
}

pub fn read_battery_capacity() -> Result<u32> {
    let val = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")?;
    val.trim()
        .parse()
        .map_err(|e| AppError::Parse(format!("battery capacity: {e}")))
}

pub fn read_battery_status() -> Result<String> {
    let val = fs::read_to_string("/sys/class/power_supply/BAT0/status")?;
    Ok(val.trim().to_string())
}

pub fn read_charge_limit() -> Result<u32> {
    let val = fs::read_to_string("/sys/class/power_supply/BAT0/charge_control_end_threshold")?;
    val.trim()
        .parse()
        .map_err(|e| AppError::Parse(format!("charge limit: {e}")))
}

/// Read all sensor data at once (cheap sysfs reads)
pub fn read_sensors(paths: &HwmonPaths) -> SensorData {
    let cpu_temp = paths
        .k10temp
        .as_ref()
        .and_then(|p| read_temp(p).ok());

    let gpu_temp = paths
        .amdgpu
        .iter()
        .filter_map(|p| read_temp(p).ok())
        .next();

    let (cpu_fan, gpu_fan) = paths
        .asus_fans
        .as_ref()
        .map(|p| {
            (
                read_fan_rpm(p, 1).ok(),
                read_fan_rpm(p, 2).ok(),
            )
        })
        .unwrap_or((None, None));

    let battery_capacity = read_battery_capacity().ok();
    let battery_status = read_battery_status().ok();
    let charge_limit = read_charge_limit().ok();

    SensorData {
        cpu_temp,
        gpu_temp,
        cpu_fan,
        gpu_fan,
        battery_capacity,
        battery_status,
        charge_limit,
    }
}

#[derive(Debug, Default)]
pub struct SensorData {
    pub cpu_temp: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub cpu_fan: Option<u32>,
    pub gpu_fan: Option<u32>,
    pub battery_capacity: Option<u32>,
    pub battery_status: Option<String>,
    pub charge_limit: Option<u32>,
}
