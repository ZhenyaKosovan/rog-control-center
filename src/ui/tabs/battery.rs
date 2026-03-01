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
            Constraint::Length(4),  // charge limit
            Constraint::Length(3),  // one-shot
            Constraint::Length(3),  // boot sound
            Constraint::Length(3),  // panel OD
            Constraint::Length(3),  // keyboard brightness
            Constraint::Min(0),    // spacer
        ])
        .split(area);

    draw_charge_limit(f, app, chunks[0]);
    draw_oneshot(f, app, chunks[1]);
    draw_toggle(f, app, chunks[2], "Boot Sound", app.state.boot_sound, 2);
    draw_toggle(f, app, chunks[3], "Panel Overdrive", app.state.panel_overdrive, 3);
    draw_kb_brightness(f, app, chunks[4]);
}

fn draw_charge_limit(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.bat_selected == 0;
    let block = Block::default()
        .title(" Charge Limit ")
        .title_style(if focused { theme::header() } else { Style::default().fg(theme::muted()) })
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let limit = app.state.charge_limit;
    let ratio = (limit as f64 - 20.0) / 80.0;
    let color = if limit >= 80 {
        theme::green()
    } else if limit >= 50 {
        theme::yellow()
    } else {
        theme::red()
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let info = Line::from(vec![
        Span::styled(
            format!(" {limit}%"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  (20-100%)  ←→ adjust", Style::default().fg(theme::muted())),
    ]);
    f.render_widget(Paragraph::new(info), rows[0]);

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

fn draw_oneshot(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.bat_selected == 1;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        if focused {
            Span::styled(" ▸ ", Style::default().fg(theme::teal()))
        } else {
            Span::styled("   ", Style::default())
        },
        Span::styled("󰂄 One-Shot Full Charge", Style::default().fg(theme::fg())),
        Span::styled(
            "  (charges to 100% once, then reverts to limit)",
            Style::default().fg(theme::muted()),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn draw_toggle(f: &mut Frame, app: &App, area: Rect, label: &str, enabled: bool, idx: usize) {
    let focused = app.bat_selected == idx;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let toggle_icon = if enabled { "◉ On " } else { "○ Off" };
    let toggle_color = if enabled { theme::green() } else { theme::muted() };

    let line = Line::from(vec![
        if focused {
            Span::styled(" ▸ ", Style::default().fg(theme::teal()))
        } else {
            Span::styled("   ", Style::default())
        },
        Span::styled(format!("{label}: "), Style::default().fg(theme::fg())),
        Span::styled(
            toggle_icon,
            Style::default().fg(toggle_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  (Enter to toggle)",
            Style::default().fg(theme::muted()),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}

fn draw_kb_brightness(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.bat_selected == 4;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if focused { theme::border_focused() } else { theme::border() });

    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = Line::from(vec![
        if focused {
            Span::styled(" ▸ ", Style::default().fg(theme::teal()))
        } else {
            Span::styled("   ", Style::default())
        },
        Span::styled("󰥻 Keyboard Brightness: ", Style::default().fg(theme::fg())),
        Span::styled(
            &app.state.keyboard_brightness,
            Style::default().fg(theme::mauve()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  (Enter to cycle)",
            Style::default().fg(theme::muted()),
        ),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}
