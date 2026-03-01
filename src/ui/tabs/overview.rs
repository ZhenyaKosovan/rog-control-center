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
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(6)])
        .split(area);

    draw_status_block(f, app, rows[0]);

    // Bottom: two columns for CPU and GPU sensors
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    draw_sensor_column(
        f,
        "CPU",
        app.state.cpu_temp,
        app.state.cpu_fan_rpm,
        &app.state.cpu_temp_history,
        &app.state.cpu_fan_history,
        theme::sky(),
        cols[0],
    );
    draw_sensor_column(
        f,
        "GPU",
        app.state.gpu_temp,
        app.state.gpu_fan_rpm,
        &app.state.gpu_temp_history,
        &app.state.gpu_fan_history,
        theme::mauve(),
        cols[1],
    );
}

fn draw_status_block(f: &mut Frame, app: &App, area: Rect) {
    let profile_color = theme::profile_color(&app.state.profile);
    let profile_icon = match app.state.profile.to_lowercase().as_str() {
        "quiet" => "󰤆",
        "balanced" => "󰾅",
        "performance" => "󰓅",
        _ => "?",
    };

    let gpu_icon = match app.state.gpu_mode.as_str() {
        "Integrated" => "󰢮",
        "Hybrid" => "󰍹",
        "AsusMuxDgpu" => "󰍺",
        _ => "?",
    };
    let gpu_color = match app.state.gpu_mode.as_str() {
        "Integrated" => theme::green(),
        "Hybrid" => theme::blue(),
        "AsusMuxDgpu" => theme::red(),
        _ => theme::fg(),
    };

    let cap = app.state.battery_capacity;
    let bat_color = if cap > 50 {
        theme::green()
    } else if cap > 20 {
        theme::yellow()
    } else {
        theme::red()
    };
    let bat_icon = match app.state.battery_status.as_str() {
        "Charging" => "󰂄",
        "Discharging" => "󰂃",
        "Full" => "󰁹",
        _ => "󰂑",
    };

    let kb_icon = "󰥻";

    let block = Block::default()
        .title(" System ")
        .title_style(Style::default().fg(theme::blue()).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border_color()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Line 1: profile + PPT | GPU mode
    let line1 = Line::from(vec![
        Span::styled(
            format!(" {profile_icon} {}", app.state.profile),
            Style::default().fg(profile_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  APU:{}W PLAT:{}W", app.state.ppt_apu, app.state.ppt_platform),
            theme::label(),
        ),
        Span::styled("  │  ", Style::default().fg(theme::border_color())),
        Span::styled(
            format!("{gpu_icon} {}", app.state.gpu_mode),
            Style::default().fg(gpu_color).add_modifier(Modifier::BOLD),
        ),
    ]);

    // Line 2: battery + keyboard + toggles
    let line2 = Line::from(vec![
        Span::styled(
            format!(" {bat_icon} {cap}%"),
            Style::default().fg(bat_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", app.state.battery_status), theme::label()),
        Span::styled(format!("  Limit:{}%", app.state.charge_limit), theme::label()),
        Span::styled("  │  ", Style::default().fg(theme::border_color())),
        Span::styled(
            format!("{kb_icon} {}", app.state.keyboard_brightness),
            Style::default().fg(theme::mauve()),
        ),
        Span::styled(
            format!(
                "  BS:{} OD:{}",
                if app.state.boot_sound { "On" } else { "Off" },
                if app.state.panel_overdrive { "On" } else { "Off" },
            ),
            theme::label(),
        ),
    ]);

    f.render_widget(Paragraph::new(vec![line1, line2]), inner);
}

fn draw_sensor_column(
    f: &mut Frame,
    label: &str,
    temp: f64,
    fan_rpm: u32,
    temp_history: &[u64],
    fan_history: &[u64],
    sparkline_color: ratatui::style::Color,
    area: Rect,
) {
    let block = Block::default()
        .title(format!(" {label} "))
        .title_style(Style::default().fg(sparkline_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::border_color()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 4 rows: temp label, temp sparkline, fan label, fan sparkline
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    // Temp label
    let temp_line = Line::from(vec![
        Span::styled(format!(" {label} "), theme::label()),
        Span::styled(format!("{temp:.0}°C"), temp_style(temp)),
    ]);
    f.render_widget(Paragraph::new(temp_line), rows[0]);

    // Temp sparkline (max 100 to pin Y-axis)
    if !temp_history.is_empty() {
        let sparkline = Sparkline::default()
            .data(temp_history)
            .max(100)
            .style(Style::default().fg(sparkline_color))
            .bar_set(symbols::bar::NINE_LEVELS);
        f.render_widget(sparkline, rows[1]);
    }

    // Fan label
    let fan_line = Line::from(vec![
        Span::styled(" Fan ", theme::label()),
        Span::styled(format!("{fan_rpm} RPM"), theme::value()),
    ]);
    f.render_widget(Paragraph::new(fan_line), rows[2]);

    // Fan sparkline (auto-scale)
    if !fan_history.is_empty() {
        let sparkline = Sparkline::default()
            .data(fan_history)
            .style(Style::default().fg(sparkline_color))
            .bar_set(symbols::bar::NINE_LEVELS);
        f.render_widget(sparkline, rows[3]);
    }
}

fn temp_style(temp: f64) -> Style {
    if temp > 85.0 {
        Style::default().fg(theme::red()).add_modifier(Modifier::BOLD)
    } else if temp > 70.0 {
        Style::default().fg(theme::yellow())
    } else {
        Style::default().fg(theme::green())
    }
}
