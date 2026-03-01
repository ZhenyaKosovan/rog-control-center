use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};

struct ThemePalette {
    bg: Color,
    fg: Color,
    surface: Color,
    border: Color,
    muted: Color,
    teal: Color,
    yellow: Color,
    red: Color,
    green: Color,
    blue: Color,
    mauve: Color,
    peach: Color,
    sky: Color,
    pink: Color,
}

static PALETTE: OnceLock<ThemePalette> = OnceLock::new();

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn load_omarchy() -> Option<ThemePalette> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{home}/.config/omarchy/current/theme/colors.toml");
    let content = std::fs::read_to_string(path).ok()?;

    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            map.insert(key.to_string(), val.to_string());
        }
    }

    Some(ThemePalette {
        bg: parse_hex(map.get("background")?)?,
        fg: parse_hex(map.get("foreground")?)?,
        surface: parse_hex(map.get("color0")?)?,
        border: parse_hex(map.get("color8")?)?,
        muted: parse_hex(map.get("color7")?)?,
        teal: parse_hex(map.get("accent")?)?,
        red: parse_hex(map.get("color1")?)?,
        green: parse_hex(map.get("color2")?)?,
        yellow: parse_hex(map.get("color3")?)?,
        blue: parse_hex(map.get("color4")?)?,
        mauve: parse_hex(map.get("color5")?)?,
        peach: parse_hex(map.get("color11")?)?,
        pink: parse_hex(map.get("color13")?)?,
        sky: parse_hex(map.get("color14")?)?,
    })
}

fn default_palette() -> ThemePalette {
    ThemePalette {
        bg: Color::Rgb(0x1a, 0x1b, 0x26),
        fg: Color::Rgb(0xa9, 0xb1, 0xd6),
        surface: Color::Rgb(0x24, 0x28, 0x3b),
        border: Color::Rgb(0x3b, 0x40, 0x61),
        muted: Color::Rgb(0x56, 0x5f, 0x89),
        teal: Color::Rgb(0x94, 0xe2, 0xd5),
        yellow: Color::Rgb(0xf9, 0xe2, 0xaf),
        red: Color::Rgb(0xf3, 0x8b, 0xa8),
        green: Color::Rgb(0xa6, 0xe3, 0xa1),
        blue: Color::Rgb(0x89, 0xb4, 0xfa),
        mauve: Color::Rgb(0xcb, 0xa6, 0xf7),
        peach: Color::Rgb(0xfa, 0xb3, 0x87),
        sky: Color::Rgb(0x89, 0xdc, 0xeb),
        pink: Color::Rgb(0xf5, 0xc2, 0xe7),
    }
}

pub fn init() {
    PALETTE.get_or_init(|| load_omarchy().unwrap_or_else(default_palette));
}

fn palette() -> &'static ThemePalette {
    PALETTE.get_or_init(|| load_omarchy().unwrap_or_else(default_palette))
}

// Color accessors
pub fn bg() -> Color { palette().bg }
pub fn fg() -> Color { palette().fg }
pub fn surface() -> Color { palette().surface }
pub fn border_color() -> Color { palette().border }
pub fn muted() -> Color { palette().muted }
pub fn teal() -> Color { palette().teal }
pub fn yellow() -> Color { palette().yellow }
pub fn red() -> Color { palette().red }
pub fn green() -> Color { palette().green }
pub fn blue() -> Color { palette().blue }
pub fn mauve() -> Color { palette().mauve }
pub fn peach() -> Color { palette().peach }
pub fn sky() -> Color { palette().sky }
pub fn pink() -> Color { palette().pink }

// Style helpers

pub fn base() -> Style {
    Style::default().fg(fg()).bg(bg())
}

pub fn header() -> Style {
    Style::default().fg(teal()).bg(bg()).add_modifier(Modifier::BOLD)
}

pub fn tab_active() -> Style {
    Style::default().fg(bg()).bg(teal()).add_modifier(Modifier::BOLD)
}

pub fn tab_inactive() -> Style {
    Style::default().fg(muted()).bg(surface())
}

pub fn border() -> Style {
    Style::default().fg(border_color())
}

pub fn border_focused() -> Style {
    Style::default().fg(teal())
}

pub fn label() -> Style {
    Style::default().fg(muted())
}

pub fn value() -> Style {
    Style::default().fg(fg()).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().fg(bg()).bg(blue())
}

pub fn status_ok() -> Style {
    Style::default().fg(green())
}

pub fn status_warn() -> Style {
    Style::default().fg(yellow())
}

pub fn status_error() -> Style {
    Style::default().fg(red())
}

pub fn footer() -> Style {
    Style::default().fg(muted()).bg(surface())
}

pub fn profile_color(profile: &str) -> Color {
    match profile.to_lowercase().as_str() {
        "quiet" => teal(),
        "balanced" => yellow(),
        "performance" => red(),
        _ => fg(),
    }
}
