pub mod asusctl;
pub mod supergfxctl;
pub mod sysfs;

use crate::app::AppAction;

#[derive(Debug, Default, Clone)]
pub struct SystemState {
    // Sensors (refreshed every 2s from sysfs)
    pub cpu_temp: f64,
    pub gpu_temp: f64,
    pub cpu_fan_rpm: u32,
    pub gpu_fan_rpm: u32,
    pub battery_capacity: u32,
    pub battery_status: String,

    // Fan RPM history for sparklines
    pub cpu_fan_history: Vec<u64>,
    pub gpu_fan_history: Vec<u64>,

    // Temperature history for sparklines
    pub cpu_temp_history: Vec<u64>,
    pub gpu_temp_history: Vec<u64>,

    // Profile & power (refreshed every 10s or on-demand)
    pub profile: String,
    pub ppt_apu: u32,
    pub ppt_platform: u32,

    // Battery
    pub charge_limit: u32,

    // GPU
    pub gpu_mode: String,
    pub gpu_modes_available: Vec<String>,

    // Keyboard
    pub keyboard_brightness: String,

    // Armoury
    pub boot_sound: bool,
    pub panel_overdrive: bool,

    // Fan curves
    pub fan_curves: Vec<asusctl::FanCurve>,

    // Board info
    pub board_name: String,
}

impl SystemState {
    /// Read fast sensor data from sysfs
    pub fn refresh_sensors(&mut self, hwmon: &sysfs::HwmonPaths) {
        let data = sysfs::read_sensors(hwmon);
        if let Some(t) = data.cpu_temp {
            self.cpu_temp = t;
            self.cpu_temp_history.push(t as u64);
            if self.cpu_temp_history.len() > 60 {
                self.cpu_temp_history.remove(0);
            }
        }
        if let Some(t) = data.gpu_temp {
            self.gpu_temp = t;
            self.gpu_temp_history.push(t as u64);
            if self.gpu_temp_history.len() > 60 {
                self.gpu_temp_history.remove(0);
            }
        }
        if let Some(rpm) = data.cpu_fan {
            self.cpu_fan_rpm = rpm;
            self.cpu_fan_history.push(rpm as u64);
            if self.cpu_fan_history.len() > 60 {
                self.cpu_fan_history.remove(0);
            }
        }
        if let Some(rpm) = data.gpu_fan {
            self.gpu_fan_rpm = rpm;
            self.gpu_fan_history.push(rpm as u64);
            if self.gpu_fan_history.len() > 60 {
                self.gpu_fan_history.remove(0);
            }
        }
        if let Some(cap) = data.battery_capacity {
            self.battery_capacity = cap;
        }
        if let Some(status) = data.battery_status {
            self.battery_status = status;
        }
        if let Some(limit) = data.charge_limit {
            self.charge_limit = limit;
        }
    }

    /// Read slower CLI data (profile, armoury, GPU mode, etc.)
    pub fn refresh_cli(&mut self) {
        if let Ok(p) = asusctl::get_profile() {
            self.profile = p;
        }
        if let Ok(mode) = supergfxctl::get_mode() {
            self.gpu_mode = mode;
        }
        if let Ok(modes) = supergfxctl::get_supported_modes() {
            self.gpu_modes_available = modes;
        }
        if let Ok(attrs) = asusctl::get_armoury_attrs() {
            self.ppt_apu = attrs.ppt_apu;
            self.ppt_platform = attrs.ppt_platform;
            self.boot_sound = attrs.boot_sound;
            self.panel_overdrive = attrs.panel_overdrive;
        }
        if let Ok(kb) = asusctl::get_keyboard_brightness() {
            self.keyboard_brightness = kb;
        }
        if let Ok(curves) = asusctl::get_fan_curves(&self.profile) {
            self.fan_curves = curves;
        }
    }

    /// Initial full load
    pub fn load_initial(&mut self, hwmon: &sysfs::HwmonPaths) {
        self.board_name = "GA402RJ".to_string();
        self.refresh_sensors(hwmon);
        self.refresh_cli();
    }
}

/// Execute an action, returning a status message
pub fn execute_action(action: &AppAction, state: &mut SystemState) -> String {
    match action {
        AppAction::SetProfile(p) => match asusctl::set_profile(p) {
            Ok(()) => {
                state.profile = p.clone();
                format!("Profile set to {p}")
            }
            Err(e) => format!("Failed to set profile: {e}"),
        },
        AppAction::SetPptApu(v) => {
            match asusctl::set_armoury("ppt_apu_sppt", &v.to_string()) {
                Ok(()) => {
                    state.ppt_apu = *v;
                    format!("APU PPT set to {v}W")
                }
                Err(e) => format!("Failed to set APU PPT: {e}"),
            }
        }
        AppAction::SetPptPlatform(v) => {
            match asusctl::set_armoury("ppt_platform_sppt", &v.to_string()) {
                Ok(()) => {
                    state.ppt_platform = *v;
                    format!("Platform PPT set to {v}W")
                }
                Err(e) => format!("Failed to set Platform PPT: {e}"),
            }
        }
        AppAction::AdjustFanPoint {
            fan,
            point,
            pwm_delta,
            temp_delta,
        } => {
            if let Some(curve) = state.fan_curves.get_mut(*fan) {
                let new_pwm = (curve.pwm[*point] as i32 + pwm_delta).clamp(0, 255) as u32;
                let new_temp = (curve.temp[*point] as i32 + temp_delta).clamp(20, 100) as u32;
                curve.pwm[*point] = new_pwm;
                curve.temp[*point] = new_temp;
                format!(
                    "{} point {}: {}°C @ {}%",
                    curve.fan,
                    point,
                    new_temp,
                    (new_pwm as f64 / 255.0 * 100.0).round()
                )
            } else {
                "No fan curve data".to_string()
            }
        }
        AppAction::ApplyFanCurve => {
            let mut msgs = Vec::new();
            let profile = state.profile.clone();
            for curve in &state.fan_curves {
                let data = curve.to_data_string();
                match asusctl::set_fan_curve(&profile, &curve.fan.to_lowercase(), &data) {
                    Ok(()) => msgs.push(format!("{} curve applied", curve.fan)),
                    Err(e) => msgs.push(format!("{} curve failed: {e}", curve.fan)),
                }
            }
            msgs.join(", ")
        }
        AppAction::ResetFanCurve => match asusctl::reset_fan_curve() {
            Ok(()) => {
                if let Ok(curves) = asusctl::get_fan_curves(&state.profile) {
                    state.fan_curves = curves;
                }
                "Fan curves reset to default".to_string()
            }
            Err(e) => format!("Failed to reset fan curves: {e}"),
        },
        AppAction::SetAuraEffect(effect) => match asusctl::set_aura_effect(effect) {
            Ok(()) => format!("Aura effect: {effect}"),
            Err(e) => format!("Failed to set aura effect: {e}"),
        },
        AppAction::SetAuraColor(color) => {
            match asusctl::set_aura_effect_with_color("static", color) {
                Ok(()) => format!("Aura color: #{color}"),
                Err(e) => format!("Failed to set aura color: {e}"),
            }
        }
        AppAction::SetGpuMode(mode) => match supergfxctl::set_mode(mode) {
            Ok(()) => {
                state.gpu_mode = mode.clone();
                if mode == "AsusMuxDgpu" {
                    "GPU mode set - REBOOT REQUIRED".to_string()
                } else {
                    format!("GPU mode: {mode}")
                }
            }
            Err(e) => format!("Failed to set GPU mode: {e}"),
        },
        AppAction::SetChargeLimit(limit) => match asusctl::set_battery_limit(*limit) {
            Ok(()) => {
                state.charge_limit = *limit;
                format!("Charge limit: {limit}%")
            }
            Err(e) => format!("Failed to set charge limit: {e}"),
        },
        AppAction::BatteryOneShot => match asusctl::battery_oneshot() {
            Ok(()) => "One-shot full charge enabled".to_string(),
            Err(e) => format!("Failed to enable one-shot: {e}"),
        },
        AppAction::ToggleBootSound => {
            let new_val = if state.boot_sound { "0" } else { "1" };
            match asusctl::set_armoury("boot_sound", new_val) {
                Ok(()) => {
                    state.boot_sound = !state.boot_sound;
                    format!(
                        "Boot sound: {}",
                        if state.boot_sound { "On" } else { "Off" }
                    )
                }
                Err(e) => format!("Failed to toggle boot sound: {e}"),
            }
        }
        AppAction::TogglePanelOd => {
            let new_val = if state.panel_overdrive { "0" } else { "1" };
            match asusctl::set_armoury("panel_overdrive", new_val) {
                Ok(()) => {
                    state.panel_overdrive = !state.panel_overdrive;
                    format!(
                        "Panel overdrive: {}",
                        if state.panel_overdrive { "On" } else { "Off" }
                    )
                }
                Err(e) => format!("Failed to toggle panel OD: {e}"),
            }
        }
        AppAction::CycleKbBrightness => match asusctl::next_keyboard_brightness() {
            Ok(()) => {
                if let Ok(kb) = asusctl::get_keyboard_brightness() {
                    state.keyboard_brightness = kb.clone();
                    format!("Keyboard brightness: {kb}")
                } else {
                    "Keyboard brightness cycled".to_string()
                }
            }
            Err(e) => format!("Failed to cycle keyboard brightness: {e}"),
        },
    }
}
