use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // fan selector
            Constraint::Min(10),   // chart
            Constraint::Length(5), // point info
        ])
        .split(area);

    draw_fan_selector(f, app, chunks[0]);
    draw_fan_chart(f, app, chunks[1]);
    draw_point_info(f, app, chunks[2]);
}

fn draw_fan_selector(f: &mut Frame, app: &App, area: Rect) {
    let fans = ["CPU", "GPU"];
    let line = Line::from(
        fans.iter()
            .enumerate()
            .flat_map(|(i, name)| {
                let style = if i == app.fan_selected_fan {
                    theme::tab_active()
                } else {
                    theme::tab_inactive()
                };
                vec![
                    Span::styled(format!(" {name} "), style),
                    Span::raw("  "),
                ]
            })
            .collect::<Vec<_>>(),
    );

    f.render_widget(Paragraph::new(line), area);
}

fn draw_fan_chart(f: &mut Frame, app: &App, area: Rect) {
    let curve = match app.state.fan_curves.get(app.fan_selected_fan) {
        Some(c) => c,
        None => {
            let msg = Paragraph::new("  No fan curve data available")
                .style(Style::default().fg(theme::MUTED));
            f.render_widget(msg, area);
            return;
        }
    };

    let fan_color = if app.fan_selected_fan == 0 {
        theme::SKY
    } else {
        theme::MAUVE
    };

    // Build data points (temp, pwm%)
    let points: Vec<(f64, f64)> = (0..8)
        .map(|i| (curve.temp[i] as f64, curve.pwm_percent(i) as f64))
        .collect();

    let dataset = Dataset::default()
        .name(format!("{} Fan", curve.fan))
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(fan_color))
        .data(&points);

    // Highlight selected point
    let selected_point = vec![points[app.fan_selected_point]];
    let highlight = Dataset::default()
        .name("Selected")
        .marker(symbols::Marker::Block)
        .graph_type(GraphType::Scatter)
        .style(
            if app.fan_editing {
                Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::PEACH)
            },
        )
        .data(&selected_point);

    let x_labels = ["20", "40", "60", "80", "100"]
        .iter()
        .map(|s| Span::styled(*s, Style::default().fg(theme::MUTED)))
        .collect::<Vec<_>>();

    let y_labels = ["0%", "25%", "50%", "75%", "100%"]
        .iter()
        .map(|s| Span::styled(*s, Style::default().fg(theme::MUTED)))
        .collect::<Vec<_>>();

    let chart = Chart::new(vec![dataset, highlight])
        .block(
            Block::default()
                .title(format!(" {} Fan Curve ", curve.fan))
                .title_style(Style::default().fg(fan_color).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(theme::border()),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("Temp (°C)", Style::default().fg(theme::MUTED)))
                .bounds([20.0, 100.0])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Fan %", Style::default().fg(theme::MUTED)))
                .bounds([0.0, 100.0])
                .labels(y_labels),
        );

    f.render_widget(chart, area);
}

fn draw_point_info(f: &mut Frame, app: &App, area: Rect) {
    let curve = match app.state.fan_curves.get(app.fan_selected_fan) {
        Some(c) => c,
        None => return,
    };

    let i = app.fan_selected_point;

    let editing_hint = if app.fan_editing {
        Span::styled(
            "  [EDITING: ↑↓ PWM, ←→ Temp, Enter: Apply, Esc: Cancel]",
            Style::default().fg(theme::YELLOW),
        )
    } else {
        Span::styled(
            "  [Press e to edit, d for defaults]",
            Style::default().fg(theme::MUTED),
        )
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  Point {}: ", i + 1),
                Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{}°C", curve.temp[i]), theme::value()),
            Span::styled(" @ ", theme::label()),
            Span::styled(format!("{}%", curve.pwm_percent(i)), theme::value()),
            Span::styled(format!(" (PWM: {})", curve.pwm[i]), theme::label()),
        ]),
        Line::from(editing_hint),
        Line::from(""),
        Line::from(
            (0..8)
                .flat_map(|j| {
                    let style = if j == i {
                        Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::MUTED)
                    };
                    vec![
                        Span::styled(
                            format!(" {}°C:{}%", curve.temp[j], curve.pwm_percent(j)),
                            style,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    ];

    f.render_widget(Paragraph::new(lines), area);
}
