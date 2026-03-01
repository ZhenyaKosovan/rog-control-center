use std::process::Command;

use crate::error::{AppError, Result};

fn run_asusctl(args: &[&str]) -> Result<String> {
    let output = Command::new("asusctl")
        .args(args)
        .output()
        .map_err(|e| AppError::Command {
            cmd: format!("asusctl {}", args.join(" ")),
            msg: e.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Command {
            cmd: format!("asusctl {}", args.join(" ")),
            msg: stderr,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_profile() -> Result<String> {
    let out = run_asusctl(&["profile", "get"])?;
    // "Active profile: Quiet\n"
    out.lines()
        .find(|l| l.starts_with("Active profile:"))
        .and_then(|l| l.strip_prefix("Active profile:"))
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AppError::Parse("Could not parse active profile".into()))
}

pub fn set_profile(profile: &str) -> Result<()> {
    run_asusctl(&["profile", "set", profile])?;
    signal_waybar();
    Ok(())
}

pub fn get_fan_curves(profile: &str) -> Result<Vec<FanCurve>> {
    let out = run_asusctl(&["fan-curve", "--mod-profile", profile])?;
    parse_fan_curves(&out)
}

#[derive(Debug, Clone)]
pub struct FanCurve {
    pub fan: String,
    pub pwm: [u32; 8],
    pub temp: [u32; 8],
    pub enabled: bool,
}

impl FanCurve {
    /// Convert PWM (0-255) to percentage (0-100)
    pub fn pwm_percent(&self, idx: usize) -> u32 {
        (self.pwm[idx] as f64 / 255.0 * 100.0).round() as u32
    }

    /// Format as asusctl data string: "30c:1%,49c:2%,..."
    pub fn to_data_string(&self) -> String {
        (0..8)
            .map(|i| format!("{}c:{}%", self.temp[i], self.pwm_percent(i)))
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn parse_fan_curves(output: &str) -> Result<Vec<FanCurve>> {
    let mut curves = Vec::new();
    let mut current_fan = String::new();
    let mut pwm = [0u32; 8];
    let mut temp = [0u32; 8];
    let mut enabled;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("fan:") {
            current_fan = line
                .strip_prefix("fan:")
                .unwrap()
                .trim()
                .trim_end_matches(',')
                .to_string();
        } else if line.starts_with("pwm:") {
            pwm = parse_tuple_values(line.strip_prefix("pwm:").unwrap())?;
        } else if line.starts_with("temp:") {
            temp = parse_tuple_values(line.strip_prefix("temp:").unwrap())?;
        } else if line.starts_with("enabled:") {
            let val = line
                .strip_prefix("enabled:")
                .unwrap()
                .trim()
                .trim_end_matches(',');
            enabled = val == "true";

            curves.push(FanCurve {
                fan: current_fan.clone(),
                pwm,
                temp,
                enabled,
            });
        }
    }

    Ok(curves)
}

fn parse_tuple_values(s: &str) -> Result<[u32; 8]> {
    // "(1, 2, 3, 56, 56, 86, 86, 104),"
    let s = s.trim().trim_end_matches(',');
    let s = s.trim_start_matches('(').trim_end_matches(')');
    let vals: Vec<u32> = s
        .split(',')
        .map(|v| {
            v.trim()
                .parse()
                .map_err(|e| AppError::Parse(format!("tuple value: {e}")))
        })
        .collect::<Result<Vec<_>>>()?;

    if vals.len() != 8 {
        return Err(AppError::Parse(format!(
            "Expected 8 values, got {}",
            vals.len()
        )));
    }

    let mut arr = [0u32; 8];
    arr.copy_from_slice(&vals);
    Ok(arr)
}

pub fn set_fan_curve(profile: &str, fan: &str, data: &str) -> Result<()> {
    run_asusctl(&[
        "fan-curve",
        "--mod-profile",
        profile,
        "--fan",
        fan,
        "--data",
        data,
    ])?;
    Ok(())
}

pub fn reset_fan_curve() -> Result<()> {
    run_asusctl(&["fan-curve", "--default"])?;
    Ok(())
}

pub fn get_keyboard_brightness() -> Result<String> {
    let out = run_asusctl(&["leds", "get"])?;
    // "Current keyboard led brightness: Low"
    out.lines()
        .find(|l| l.contains("brightness"))
        .and_then(|l| l.rsplit(':').next())
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AppError::Parse("Could not parse keyboard brightness".into()))
}

pub fn next_keyboard_brightness() -> Result<()> {
    run_asusctl(&["leds", "next"])?;
    Ok(())
}

pub fn get_battery_info() -> Result<u32> {
    let out = run_asusctl(&["battery", "info"])?;
    // "Current battery charge limit: 80%"
    out.lines()
        .find(|l| l.contains("charge limit"))
        .and_then(|l| {
            l.rsplit(':')
                .next()
                .map(|s| s.trim().trim_end_matches('%').trim())
        })
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::Parse("Could not parse battery charge limit".into()))
}

pub fn set_battery_limit(limit: u32) -> Result<()> {
    run_asusctl(&["battery", "limit", &limit.to_string()])?;
    signal_waybar();
    Ok(())
}

pub fn battery_oneshot() -> Result<()> {
    run_asusctl(&["battery", "oneshot"])?;
    Ok(())
}

pub fn get_armoury_attrs() -> Result<ArmouryAttrs> {
    let out = run_asusctl(&["armoury", "list"])?;
    parse_armoury(&out)
}

#[derive(Debug, Default, Clone)]
pub struct ArmouryAttrs {
    pub boot_sound: bool,
    pub panel_overdrive: bool,
    pub ppt_apu: u32,
    pub ppt_platform: u32,
}

fn parse_armoury(output: &str) -> Result<ArmouryAttrs> {
    let mut attrs = ArmouryAttrs::default();
    let mut current_attr = String::new();

    for line in output.lines() {
        let trimmed = line.trim();

        if !trimmed.is_empty() && !trimmed.starts_with("current:") && trimmed.ends_with(':') {
            current_attr = trimmed.trim_end_matches(':').to_string();
        } else if trimmed.starts_with("current:") {
            let val = trimmed.strip_prefix("current:").unwrap().trim();
            match current_attr.as_str() {
                "boot_sound" => {
                    // "[(0),1]" means 0 is selected, so boot_sound is off
                    // "[0,(1)]" means 1 is selected, so boot_sound is on
                    attrs.boot_sound = val.contains("(1)");
                }
                "panel_overdrive" => {
                    attrs.panel_overdrive = val.contains("(1)");
                }
                "ppt_apu_sppt" => {
                    // "15..[15]..80" - value in brackets
                    if let Some(v) = extract_bracket_value(val) {
                        attrs.ppt_apu = v;
                    }
                }
                "ppt_platform_sppt" => {
                    if let Some(v) = extract_bracket_value(val) {
                        attrs.ppt_platform = v;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(attrs)
}

fn extract_bracket_value(s: &str) -> Option<u32> {
    // "15..[15]..80"
    let start = s.find('[')?;
    let end = s.find(']')?;
    s[start + 1..end].parse().ok()
}

pub fn set_armoury(attr: &str, value: &str) -> Result<()> {
    run_asusctl(&["armoury", "set", attr, value])?;
    Ok(())
}

pub fn set_aura_effect(effect: &str) -> Result<()> {
    run_asusctl(&["aura", "effect", effect])?;
    Ok(())
}

pub fn set_aura_effect_with_color(effect: &str, color: &str) -> Result<()> {
    // Color format: "#rrggbb" or "rrggbb"
    let color = color.trim_start_matches('#');
    let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(255);

    run_asusctl(&[
        "aura", "effect", effect,
        "-c1", &format!("{r},{g},{b}"),
    ])?;
    Ok(())
}

fn signal_waybar() {
    let _ = Command::new("pkill")
        .args(["-RTMIN+11", "waybar"])
        .spawn();
}
