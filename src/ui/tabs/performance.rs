use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // profile selector
            Constraint::Length(4),  // APU PPT
            Constraint::Length(4),  // Platform PPT
            Constraint::Min(0),    // spacer
        ])
        .split(area);

    draw_profile_selector(f, app, chunks[0]);
    draw_ppt_slider(f, app, chunks[1], "APU PPT (SPPT)", app.state.ppt_apu, 15, 80, 1);
    draw_ppt_slider(f, app, chunks[2], "Platform PPT (SPPT)", app.state.ppt_platform, 30, 115, 2);
}

fn draw_profile_selector(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.perf_selected == 0;
    let block = Block::default()
        .title(" Profile ")
        .title_style(if focused { theme::header() } else { Style::default().fg(theme::muted()) })
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let profiles = ["Quiet", "Balanced", "Performance"];
    let icons = ["󰤆", "󰾅", "󰓅"];

    let lines: Vec<Line> = profiles
        .iter()
        .zip(icons.iter())
        .map(|(name, icon)| {
            let is_active = app.state.profile == *name;
            let color = theme::profile_color(name);

            let marker = if is_active { "◉" } else { "○" };
            let style = if is_active {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::fg())
            };

            Line::from(vec![
                Span::styled(format!("  {marker} {icon} "), style),
                Span::styled(*name, style),
                if is_active {
                    Span::styled(" ◀", Style::default().fg(color))
                } else {
                    Span::raw("")
                },
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_ppt_slider(
    f: &mut Frame,
    app: &App,
    area: Rect,
    title: &str,
    value: u32,
    min: u32,
    max: u32,
    idx: usize,
) {
    let focused = app.perf_selected == idx;
    let block = Block::default()
        .title(format!(" {title} "))
        .title_style(if focused { theme::header() } else { Style::default().fg(theme::muted()) })
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let ratio = (value - min) as f64 / (max - min) as f64;
    let color = if ratio > 0.8 {
        theme::red()
    } else if ratio > 0.5 {
        theme::yellow()
    } else {
        theme::green()
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let info_line = Line::from(vec![
        Span::styled(
            format!(" {value}W"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  ({min}-{max}W)  ←→ adjust"), Style::default().fg(theme::muted())),
    ]);
    f.render_widget(Paragraph::new(info_line), rows[0]);

    let track_width = rows[1].width.saturating_sub(2) as usize;
    let marker_pos = ((ratio * track_width as f64) as usize).min(track_width.saturating_sub(1));
    let before = "─".repeat(marker_pos);
    let after = "─".repeat(track_width.saturating_sub(marker_pos + 1));

    let track_line = Line::from(vec![
        Span::raw(" "),
        Span::styled(before, Style::default().fg(color)),
        Span::styled("●", Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(after, Style::default().fg(theme::surface())),
    ]);
    f.render_widget(Paragraph::new(track_line), rows[1]);
}
