use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::theme;

const EFFECTS: &[(&str, &str)] = &[
    ("static", "Static single color"),
    ("breathe", "Two-color breathing"),
    ("rainbow-cycle", "Rainbow cycle"),
    ("rainbow-wave", "Rainbow wave"),
    ("stars", "Twinkling stars"),
    ("rain", "Rain drops"),
    ("highlight", "Highlight sweep"),
    ("laser", "Laser beam"),
    ("ripple", "Ripple effect"),
    ("pulse", "Pulsing glow"),
    ("comet", "Comet trail"),
    ("flash", "Flash strobe"),
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_effect_list(f, app, chunks[0]);
    draw_color_panel(f, app, chunks[1]);
}

fn draw_effect_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Effects ")
        .title_style(Style::default().fg(theme::mauve()).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let items: Vec<ListItem> = EFFECTS
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let style = if i == app.aura_selected {
                theme::selected()
            } else {
                Style::default().fg(theme::fg())
            };

            let marker = if i == app.aura_selected { "▸" } else { " " };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {marker} "), style),
                Span::styled(format!("{:<16}", name), style),
                Span::styled(
                    *desc,
                    if i == app.aura_selected {
                        style
                    } else {
                        Style::default().fg(theme::muted())
                    },
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_color_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Settings ")
        .title_style(Style::default().fg(theme::pink()).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let color_display = if app.aura_editing_color {
        format!("#{}_", app.aura_color_input)
    } else {
        "Press 'c' to enter color".to_string()
    };

    let color_style = if app.aura_editing_color {
        Style::default().fg(theme::yellow()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::muted())
    };

    // Preview color swatch if we have a valid color
    let swatch = if app.aura_color_input.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&app.aura_color_input[0..2], 16),
            u8::from_str_radix(&app.aura_color_input[2..4], 16),
            u8::from_str_radix(&app.aura_color_input[4..6], 16),
        ) {
            Some(ratatui::style::Color::Rgb(r, g, b))
        } else {
            None
        }
    } else {
        None
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Color", Style::default().fg(theme::fg()).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(color_display, color_style),
        ]),
    ];

    if let Some(color) = swatch {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("████████", Style::default().fg(color)),
        ]));
    }

    lines.extend(vec![
        Line::from(""),
        Line::from(Span::styled("  Shortcuts", Style::default().fg(theme::fg()).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  ↑↓      ", Style::default().fg(theme::teal())),
            Span::raw("Select effect"),
        ]),
        Line::from(vec![
            Span::styled("  Enter   ", Style::default().fg(theme::teal())),
            Span::raw("Apply effect"),
        ]),
        Line::from(vec![
            Span::styled("  c       ", Style::default().fg(theme::teal())),
            Span::raw("Enter hex color"),
        ]),
        Line::from(vec![
            Span::styled("  Esc     ", Style::default().fg(theme::teal())),
            Span::raw("Cancel input"),
        ]),
    ]);

    f.render_widget(Paragraph::new(lines), inner);
}
