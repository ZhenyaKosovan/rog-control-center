use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::backend::SystemState;
use crate::ui::tabs::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
    Confirm,
}

pub struct App {
    pub running: bool,
    pub tab: Tab,
    pub state: SystemState,
    pub input_mode: InputMode,
    pub status_msg: Option<String>,
    pub show_help: bool,

    // Performance tab
    pub perf_selected: usize, // 0=profile list, 1=APU slider, 2=Platform slider

    // Fan tab
    pub fan_selected_fan: usize, // 0=CPU, 1=GPU
    pub fan_selected_point: usize,
    pub fan_editing: bool,

    // Aura tab
    pub aura_selected: usize,
    pub aura_color_input: String,
    pub aura_editing_color: bool,

    // GPU tab
    pub gpu_selected: usize,
    pub gpu_confirm: bool,

    // Battery tab
    pub bat_selected: usize,

    // Tick counter for refresh scheduling
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            tab: Tab::Overview,
            state: SystemState::default(),
            input_mode: InputMode::Normal,
            status_msg: None,
            show_help: false,
            perf_selected: 0,
            fan_selected_fan: 0,
            fan_selected_point: 0,
            fan_editing: false,
            aura_selected: 0,
            aura_color_input: String::new(),
            aura_editing_color: false,
            gpu_selected: 0,
            gpu_confirm: false,
            bat_selected: 0,
            tick_count: 0,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        // Global keys
        if self.show_help {
            self.show_help = false;
            return None;
        }

        if self.gpu_confirm {
            return self.handle_gpu_confirm(key);
        }

        if self.aura_editing_color {
            return self.handle_aura_color_input(key);
        }

        if self.fan_editing {
            return self.handle_fan_editing(key);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
                return None;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
                return None;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return None;
            }
            KeyCode::Tab => {
                self.tab = self.tab.next();
                self.input_mode = InputMode::Normal;
                return None;
            }
            KeyCode::BackTab => {
                self.tab = self.tab.prev();
                self.input_mode = InputMode::Normal;
                return None;
            }
            KeyCode::Char(c @ '1'..='6') => {
                if let Some(tab) = Tab::from_number(c as u8 - b'0') {
                    self.tab = tab;
                    self.input_mode = InputMode::Normal;
                }
                return None;
            }
            _ => {}
        }

        // Tab-specific keys
        match self.tab {
            Tab::Overview => None,
            Tab::Performance => self.handle_perf_key(key),
            Tab::Fans => self.handle_fan_key(key),
            Tab::Aura => self.handle_aura_key(key),
            Tab::Gpu => self.handle_gpu_key(key),
            Tab::Battery => self.handle_bat_key(key),
        }
    }

    fn handle_perf_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.perf_selected = self.perf_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.perf_selected < 2 {
                    self.perf_selected += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                return self.perf_adjust(-1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                return self.perf_adjust(1);
            }
            KeyCode::Enter => {
                return self.perf_apply();
            }
            _ => {}
        }
        None
    }

    fn perf_adjust(&mut self, delta: i32) -> Option<AppAction> {
        match self.perf_selected {
            0 => {
                let profiles = ["Quiet", "Balanced", "Performance"];
                let cur = profiles
                    .iter()
                    .position(|&p| p == self.state.profile)
                    .unwrap_or(0) as i32;
                let next = (cur + delta).clamp(0, 2) as usize;
                return Some(AppAction::SetProfile(profiles[next].to_string()));
            }
            1 => {
                let v = (self.state.ppt_apu as i32 + delta * 5).clamp(15, 80) as u32;
                return Some(AppAction::SetPptApu(v));
            }
            2 => {
                let v = (self.state.ppt_platform as i32 + delta * 5).clamp(30, 115) as u32;
                return Some(AppAction::SetPptPlatform(v));
            }
            _ => {}
        }
        None
    }

    fn perf_apply(&self) -> Option<AppAction> {
        match self.perf_selected {
            0 => Some(AppAction::SetProfile(self.state.profile.clone())),
            1 => Some(AppAction::SetPptApu(self.state.ppt_apu)),
            2 => Some(AppAction::SetPptPlatform(self.state.ppt_platform)),
            _ => None,
        }
    }

    fn handle_fan_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Tab => {
                self.fan_selected_fan = 1 - self.fan_selected_fan;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.fan_selected_point = self.fan_selected_point.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.fan_selected_point < 7 {
                    self.fan_selected_point += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char('e') => {
                self.fan_editing = true;
                self.input_mode = InputMode::Editing;
            }
            KeyCode::Char('d') => {
                return Some(AppAction::ResetFanCurve);
            }
            _ => {}
        }
        None
    }

    fn handle_fan_editing(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                return Some(AppAction::AdjustFanPoint {
                    fan: self.fan_selected_fan,
                    point: self.fan_selected_point,
                    pwm_delta: 5,
                    temp_delta: 0,
                });
            }
            KeyCode::Down | KeyCode::Char('j') => {
                return Some(AppAction::AdjustFanPoint {
                    fan: self.fan_selected_fan,
                    point: self.fan_selected_point,
                    pwm_delta: -5,
                    temp_delta: 0,
                });
            }
            KeyCode::Right | KeyCode::Char('l') => {
                return Some(AppAction::AdjustFanPoint {
                    fan: self.fan_selected_fan,
                    point: self.fan_selected_point,
                    pwm_delta: 0,
                    temp_delta: 2,
                });
            }
            KeyCode::Left | KeyCode::Char('h') => {
                return Some(AppAction::AdjustFanPoint {
                    fan: self.fan_selected_fan,
                    point: self.fan_selected_point,
                    pwm_delta: 0,
                    temp_delta: -2,
                });
            }
            KeyCode::Esc => {
                self.fan_editing = false;
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.fan_editing = false;
                self.input_mode = InputMode::Normal;
                return Some(AppAction::ApplyFanCurve);
            }
            _ => {}
        }
        None
    }

    fn handle_aura_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.aura_selected = self.aura_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.aura_selected += 1;
            }
            KeyCode::Enter => {
                let effects = [
                    "static", "breathe", "rainbow-cycle", "rainbow-wave", "stars", "rain",
                    "highlight", "laser", "ripple", "pulse", "comet", "flash",
                ];
                if self.aura_selected < effects.len() {
                    return Some(AppAction::SetAuraEffect(effects[self.aura_selected].to_string()));
                }
            }
            KeyCode::Char('c') => {
                self.aura_editing_color = true;
                self.aura_color_input.clear();
                self.input_mode = InputMode::Editing;
            }
            _ => {}
        }
        None
    }

    fn handle_aura_color_input(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_hexdigit() && self.aura_color_input.len() < 6 => {
                self.aura_color_input.push(c);
            }
            KeyCode::Backspace => {
                self.aura_color_input.pop();
            }
            KeyCode::Enter => {
                let color = self.aura_color_input.clone();
                self.aura_editing_color = false;
                self.input_mode = InputMode::Normal;
                if color.len() == 6 {
                    return Some(AppAction::SetAuraColor(color));
                }
            }
            KeyCode::Esc => {
                self.aura_editing_color = false;
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        None
    }

    fn handle_gpu_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.gpu_selected = self.gpu_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.gpu_selected < 2 {
                    self.gpu_selected += 1;
                }
            }
            KeyCode::Enter => {
                let modes = ["Integrated", "Hybrid", "AsusMuxDgpu"];
                let target = modes[self.gpu_selected];
                if target == "AsusMuxDgpu" || self.state.gpu_mode == "AsusMuxDgpu" {
                    self.gpu_confirm = true;
                    self.input_mode = InputMode::Confirm;
                } else {
                    return Some(AppAction::SetGpuMode(target.to_string()));
                }
            }
            _ => {}
        }
        None
    }

    fn handle_gpu_confirm(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.gpu_confirm = false;
                self.input_mode = InputMode::Normal;
                let modes = ["Integrated", "Hybrid", "AsusMuxDgpu"];
                return Some(AppAction::SetGpuMode(modes[self.gpu_selected].to_string()));
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.gpu_confirm = false;
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        None
    }

    fn handle_bat_key(&mut self, key: KeyEvent) -> Option<AppAction> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.bat_selected = self.bat_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.bat_selected < 4 {
                    self.bat_selected += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.bat_selected == 0 {
                    let v = self.state.charge_limit.saturating_sub(5).max(20);
                    return Some(AppAction::SetChargeLimit(v));
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.bat_selected == 0 {
                    let v = (self.state.charge_limit + 5).min(100);
                    return Some(AppAction::SetChargeLimit(v));
                }
            }
            KeyCode::Enter => {
                return match self.bat_selected {
                    0 => Some(AppAction::SetChargeLimit(self.state.charge_limit)),
                    1 => Some(AppAction::BatteryOneShot),
                    2 => Some(AppAction::ToggleBootSound),
                    3 => Some(AppAction::TogglePanelOd),
                    4 => Some(AppAction::CycleKbBrightness),
                    _ => None,
                };
            }
            _ => {}
        }
        None
    }
}

#[derive(Debug)]
pub enum AppAction {
    SetProfile(String),
    SetPptApu(u32),
    SetPptPlatform(u32),
    AdjustFanPoint {
        fan: usize,
        point: usize,
        pwm_delta: i32,
        temp_delta: i32,
    },
    ApplyFanCurve,
    ResetFanCurve,
    SetAuraEffect(String),
    SetAuraColor(String),
    SetGpuMode(String),
    SetChargeLimit(u32),
    BatteryOneShot,
    ToggleBootSound,
    TogglePanelOd,
    CycleKbBrightness,
}
