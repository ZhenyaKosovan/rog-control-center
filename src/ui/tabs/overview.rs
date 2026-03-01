use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // 2x3 grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[0]);

    let bot_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[1]);

    draw_profile_block(f, app, top_cols[0]);
    draw_gpu_block(f, app, top_cols[1]);
    draw_battery_block(f, app, top_cols[2]);
    draw_fan_block(f, app, bot_cols[0]);
    draw_keyboard_block(f, app, bot_cols[1]);
    draw_temps_block(f, app, bot_cols[2]);
}

fn draw_profile_block(f: &mut Frame, app: &App, area: Rect) {
    let color = theme::profile_color(&app.state.profile);
    let icon = match app.state.profile.to_lowercase().as_str() {
        "quiet" => "󰤆",
        "balanced" => "󰾅",
        "performance" => "󰓅",
        _ => "?",
    };

    let block = Block::default()
        .title(" Profile ")
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {icon} {}", app.state.profile),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  APU:  ", theme::label()),
            Span::styled(format!("{}W", app.state.ppt_apu), theme::value()),
        ]),
        Line::from(vec![
            Span::styled("  PLAT: ", theme::label()),
            Span::styled(format!("{}W", app.state.ppt_platform), theme::value()),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_gpu_block(f: &mut Frame, app: &App, area: Rect) {
    let mode_color = match app.state.gpu_mode.as_str() {
        "Integrated" => theme::GREEN,
        "Hybrid" => theme::BLUE,
        "AsusMuxDgpu" => theme::RED,
        _ => theme::FG,
    };

    let icon = match app.state.gpu_mode.as_str() {
        "Integrated" => "󰢮",
        "Hybrid" => "󰍹",
        "AsusMuxDgpu" => "󰍺",
        _ => "?",
    };

    let block = Block::default()
        .title(" GPU Mode ")
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {icon} {}", app.state.gpu_mode),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GPU Temp: ", theme::label()),
            Span::styled(format!("{:.0}°C", app.state.gpu_temp), temp_style(app.state.gpu_temp)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_battery_block(f: &mut Frame, app: &App, area: Rect) {
    let cap = app.state.battery_capacity;
    let bat_color = if cap > 50 {
        theme::GREEN
    } else if cap > 20 {
        theme::YELLOW
    } else {
        theme::RED
    };

    let status_icon = match app.state.battery_status.as_str() {
        "Charging" => "󰂄",
        "Discharging" => "󰂃",
        "Full" => "󰁹",
        _ => "󰂑",
    };

    let block = Block::default()
        .title(" Battery ")
        .title_style(Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let bar_width = (inner.width as u32).saturating_sub(6);
    let filled = (cap as f64 / 100.0 * bar_width as f64) as usize;
    let empty = bar_width as usize - filled;
    let bar = format!("  {}{}",
        "█".repeat(filled),
        "░".repeat(empty),
    );

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {status_icon} "), Style::default().fg(bat_color)),
            Span::styled(format!("{cap}%"), Style::default().fg(bat_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {}", app.state.battery_status), theme::label()),
        ]),
        Line::from(Span::styled(bar, Style::default().fg(bat_color))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Limit: ", theme::label()),
            Span::styled(format!("{}%", app.state.charge_limit), theme::value()),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_fan_block(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Fans ")
        .title_style(Style::default().fg(theme::SKY).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let fan_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    // CPU fan
    let cpu_line = Line::from(vec![
        Span::styled("  CPU: ", theme::label()),
        Span::styled(format!("{} RPM", app.state.cpu_fan_rpm), theme::value()),
    ]);
    f.render_widget(Paragraph::new(cpu_line), fan_chunks[0]);

    if !app.state.cpu_fan_history.is_empty() {
        let sparkline = Sparkline::default()
            .data(&app.state.cpu_fan_history)
            .style(Style::default().fg(theme::SKY))
            .bar_set(symbols::bar::NINE_LEVELS);
        f.render_widget(sparkline, fan_chunks[1]);
    }

    // GPU fan
    let gpu_line = Line::from(vec![
        Span::styled("  GPU: ", theme::label()),
        Span::styled(format!("{} RPM", app.state.gpu_fan_rpm), theme::value()),
    ]);
    f.render_widget(Paragraph::new(gpu_line), fan_chunks[2]);

    if !app.state.gpu_fan_history.is_empty() {
        let sparkline = Sparkline::default()
            .data(&app.state.gpu_fan_history)
            .style(Style::default().fg(theme::MAUVE))
            .bar_set(symbols::bar::NINE_LEVELS);
        f.render_widget(sparkline, fan_chunks[3]);
    }
}

fn draw_keyboard_block(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Keyboard ")
        .title_style(Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let brightness_icon = match app.state.keyboard_brightness.to_lowercase().as_str() {
        "off" => "󰌌",
        "low" => "󰥻",
        "med" => "󰥻",
        "high" => "󰥻",
        _ => "󰌌",
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {brightness_icon} Brightness: "), theme::label()),
            Span::styled(&app.state.keyboard_brightness, theme::value()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Boot Sound: ", theme::label()),
            Span::styled(
                if app.state.boot_sound { "On" } else { "Off" },
                if app.state.boot_sound { theme::status_ok() } else { theme::label() },
            ),
        ]),
        Line::from(vec![
            Span::styled("  Panel OD:   ", theme::label()),
            Span::styled(
                if app.state.panel_overdrive { "On" } else { "Off" },
                if app.state.panel_overdrive { theme::status_ok() } else { theme::label() },
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_temps_block(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Temperatures ")
        .title_style(Style::default().fg(theme::PEACH).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let cpu_temp = app.state.cpu_temp;
    let gpu_temp = app.state.gpu_temp;

    let cpu_bar_width = (inner.width as f64 * 0.6) as usize;
    let cpu_filled = ((cpu_temp / 100.0) * cpu_bar_width as f64).min(cpu_bar_width as f64) as usize;
    let gpu_filled = ((gpu_temp / 100.0) * cpu_bar_width as f64).min(cpu_bar_width as f64) as usize;

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  CPU ", theme::label()),
            Span::styled(
                "█".repeat(cpu_filled),
                temp_style(cpu_temp),
            ),
            Span::styled(
                "░".repeat(cpu_bar_width.saturating_sub(cpu_filled)),
                Style::default().fg(theme::SURFACE),
            ),
            Span::styled(format!(" {cpu_temp:.0}°C"), temp_style(cpu_temp)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  GPU ", theme::label()),
            Span::styled(
                "█".repeat(gpu_filled),
                temp_style(gpu_temp),
            ),
            Span::styled(
                "░".repeat(cpu_bar_width.saturating_sub(gpu_filled)),
                Style::default().fg(theme::SURFACE),
            ),
            Span::styled(format!(" {gpu_temp:.0}°C"), temp_style(gpu_temp)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn temp_style(temp: f64) -> Style {
    if temp > 85.0 {
        Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)
    } else if temp > 70.0 {
        Style::default().fg(theme::YELLOW)
    } else {
        Style::default().fg(theme::GREEN)
    }
}
