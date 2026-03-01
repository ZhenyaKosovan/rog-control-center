use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme;

struct GpuMode {
    name: &'static str,
    icon: &'static str,
    desc: &'static str,
    color: ratatui::style::Color,
}

fn gpu_modes() -> [GpuMode; 3] {
    [
        GpuMode {
            name: "Integrated",
            icon: "󰢮",
            desc: "iGPU only. Best battery life. dGPU is powered off.",
            color: theme::green(),
        },
        GpuMode {
            name: "Hybrid",
            icon: "󰍹",
            desc: "Auto-switch between iGPU and dGPU. Balanced performance.",
            color: theme::blue(),
        },
        GpuMode {
            name: "AsusMuxDgpu",
            icon: "󰍺",
            desc: "dGPU direct via MUX switch. Max performance. REQUIRES REBOOT.",
            color: theme::red(),
        },
    ]
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(area);

    for (i, mode) in gpu_modes().iter().enumerate() {
        draw_gpu_card(f, app, chunks[i], mode, i);
    }

    if app.gpu_confirm {
        draw_confirm_dialog(f, app);
    }
}

fn draw_gpu_card(f: &mut Frame, app: &App, area: Rect, mode: &GpuMode, idx: usize) {
    let is_active = app.state.gpu_mode == mode.name;
    let is_selected = app.gpu_selected == idx;

    let border_style = if is_selected {
        theme::border_focused()
    } else if is_active {
        Style::default().fg(mode.color)
    } else {
        theme::border()
    };

    let block = Block::default()
        .title(format!(" {} {} ", mode.icon, mode.name))
        .title_style(
            Style::default()
                .fg(if is_active || is_selected {
                    mode.color
                } else {
                    theme::muted()
                })
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let status = if is_active { "● ACTIVE" } else { "○" };
    let status_style = if is_active {
        Style::default().fg(mode.color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::muted())
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("  {status} "), status_style),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(mode.desc, Style::default().fg(theme::muted())),
        ]),
        if is_selected && !is_active {
            Line::from(Span::styled(
                "  Press Enter to switch",
                Style::default().fg(theme::teal()),
            ))
        } else {
            Line::from("")
        },
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_confirm_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let modes = ["Integrated", "Hybrid", "AsusMuxDgpu"];
    let target = modes[app.gpu_selected];

    let is_mux = target == "AsusMuxDgpu" || app.state.gpu_mode == "AsusMuxDgpu";

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ⚠ GPU Mode Change",
            Style::default().fg(theme::yellow()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Switch to {target}?"),
            Style::default().fg(theme::fg()),
        )),
        if is_mux {
            Line::from(Span::styled(
                "  MUX switch requires a REBOOT to take effect!",
                Style::default().fg(theme::red()).add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from("")
        },
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Y]", Style::default().fg(theme::green()).add_modifier(Modifier::BOLD)),
            Span::raw(" Confirm    "),
            Span::styled("[N/Esc]", Style::default().fg(theme::red()).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]),
    ];

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .title(" Confirm ")
            .title_style(theme::header())
            .borders(Borders::ALL)
            .border_style(theme::border_focused())
            .style(theme::base()),
    );

    f.render_widget(dialog, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
