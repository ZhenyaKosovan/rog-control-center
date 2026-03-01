pub mod tabs;
pub mod theme;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs as TabsWidget},
    Frame,
};

use crate::app::App;
use tabs::Tab;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(2), // tabs
            Constraint::Min(10),   // content
            Constraint::Length(1), // status/footer
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    draw_content(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);

    if app.show_help {
        draw_help_overlay(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let cpu_temp = format!("CPU:{:.0}°C", app.state.cpu_temp);
    let gpu_temp = format!("GPU:{:.0}°C", app.state.gpu_temp);

    let header = Line::from(vec![
        Span::styled(" ROG Control Center ", theme::header()),
        Span::styled("─── ", Style::default().fg(theme::border_color())),
        Span::styled(&app.state.board_name, Style::default().fg(theme::muted())),
        Span::styled(" ── ", Style::default().fg(theme::border_color())),
        Span::styled(cpu_temp, Style::default().fg(theme::peach())),
        Span::raw(" "),
        Span::styled(gpu_temp, Style::default().fg(theme::peach())),
        Span::raw(" "),
    ]);

    f.render_widget(Paragraph::new(header).style(theme::base()), area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                theme::tab_active()
            } else {
                theme::tab_inactive()
            };
            Line::from(Span::styled(format!(" {} ", t.title()), style))
        })
        .collect();

    let idx = Tab::ALL.iter().position(|&t| t == app.tab).unwrap_or(0);

    let tabs = TabsWidget::new(titles)
        .select(idx)
        .style(theme::base())
        .highlight_style(theme::tab_active())
        .divider(Span::styled("│", Style::default().fg(theme::border_color())));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .style(theme::base());

    let inner = block.inner(area);
    f.render_widget(block, area);

    match app.tab {
        Tab::Overview => tabs::overview::draw(f, app, inner),
        Tab::Performance => tabs::performance::draw(f, app, inner),
        Tab::Fans => tabs::fans::draw(f, app, inner),
        Tab::Aura => tabs::aura::draw(f, app, inner),
        Tab::Gpu => tabs::gpu::draw(f, app, inner),
        Tab::Battery => tabs::battery::draw(f, app, inner),
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = if let Some(ref msg) = app.status_msg {
        Line::from(vec![
            Span::styled(" ", theme::footer()),
            Span::styled(msg.as_str(), Style::default().fg(theme::green())),
        ])
    } else {
        let keys = match app.tab {
            Tab::Overview => "q:Quit  Tab:Next  1-6:Jump  ?:Help",
            Tab::Performance => "q:Quit  ↑↓:Select  ←→:Adjust  Enter:Apply  ?:Help",
            Tab::Fans => "q:Quit  Tab:CPU/GPU  ←→:Point  e:Edit  d:Default  ?:Help",
            Tab::Aura => "q:Quit  ↑↓:Effect  Enter:Apply  c:Color  ?:Help",
            Tab::Gpu => "q:Quit  ↑↓:Select  Enter:Apply  ?:Help",
            Tab::Battery => "q:Quit  ↑↓:Select  ←→:Adjust  Enter:Apply  ?:Help",
        };
        Line::from(Span::styled(format!(" {keys}"), theme::footer()))
    };

    f.render_widget(Paragraph::new(footer_text).style(theme::footer()), area);
}

fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            " Keyboard Shortcuts ",
            theme::header(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q / Ctrl+C  ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("  Tab         ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab   ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  1-6         ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Jump to tab"),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓ / jk     ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Navigate items"),
        ]),
        Line::from(vec![
            Span::styled("  ←→ / hl     ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Adjust values"),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Apply / Select"),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Cancel / Back"),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(theme::teal()).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Press any key to close ",
            Style::default().fg(theme::muted()),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(theme::header())
                .borders(Borders::ALL)
                .border_style(theme::border_focused())
                .style(theme::base()),
        );

    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}
