use ratatui::style::{Color, Modifier, Style};

// Tokyo Night base
pub const BG: Color = Color::Rgb(0x1a, 0x1b, 0x26);
pub const FG: Color = Color::Rgb(0xa9, 0xb1, 0xd6);
pub const SURFACE: Color = Color::Rgb(0x24, 0x28, 0x3b);
pub const BORDER: Color = Color::Rgb(0x3b, 0x40, 0x61);
pub const MUTED: Color = Color::Rgb(0x56, 0x5f, 0x89);

// Catppuccin Mocha accents
pub const TEAL: Color = Color::Rgb(0x94, 0xe2, 0xd5);
pub const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
pub const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);
pub const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
pub const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
pub const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);
pub const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
pub const SKY: Color = Color::Rgb(0x89, 0xdc, 0xeb);
pub const PINK: Color = Color::Rgb(0xf5, 0xc2, 0xe7);

pub fn base() -> Style {
    Style::default().fg(FG).bg(BG)
}

pub fn header() -> Style {
    Style::default().fg(TEAL).bg(BG).add_modifier(Modifier::BOLD)
}

pub fn tab_active() -> Style {
    Style::default().fg(BG).bg(TEAL).add_modifier(Modifier::BOLD)
}

pub fn tab_inactive() -> Style {
    Style::default().fg(MUTED).bg(SURFACE)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

pub fn border_focused() -> Style {
    Style::default().fg(TEAL)
}

pub fn label() -> Style {
    Style::default().fg(MUTED)
}

pub fn value() -> Style {
    Style::default().fg(FG).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().fg(BG).bg(BLUE)
}

pub fn status_ok() -> Style {
    Style::default().fg(GREEN)
}

pub fn status_warn() -> Style {
    Style::default().fg(YELLOW)
}

pub fn status_error() -> Style {
    Style::default().fg(RED)
}

pub fn footer() -> Style {
    Style::default().fg(MUTED).bg(SURFACE)
}

pub fn profile_color(profile: &str) -> Color {
    match profile.to_lowercase().as_str() {
        "quiet" => TEAL,
        "balanced" => YELLOW,
        "performance" => RED,
        _ => FG,
    }
}
