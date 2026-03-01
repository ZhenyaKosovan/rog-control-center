#![allow(dead_code)]

mod app;
mod backend;
mod error;
mod event;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use backend::sysfs::HwmonPaths;
use event::{Event, EventHandler};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App init
    let mut app = App::new();
    let hwmon = HwmonPaths::discover();
    app.state.load_initial(&hwmon);

    let mut events = EventHandler::new(250); // 250ms tick

    // Main loop
    let result = run_loop(&mut terminal, &mut app, &mut events, &hwmon).await;

    // Terminal teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
    hwmon: &HwmonPaths,
) -> color_eyre::Result<()> {
    while app.running {
        // Draw
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        match events.next().await {
            Some(Event::Key(key)) => {
                // Only handle key press events (not release/repeat on some terminals)
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if let Some(action) = app.handle_key(key) {
                        let msg = backend::execute_action(&action, &mut app.state);
                        app.set_status(msg);
                    }
                }
            }
            Some(Event::Tick) => {
                app.tick_count += 1;

                // Sensor refresh every ~2s (8 ticks * 250ms)
                if app.tick_count % 8 == 0 {
                    app.state.refresh_sensors(hwmon);
                }

                // CLI refresh every ~10s (40 ticks * 250ms)
                if app.tick_count % 40 == 0 {
                    app.state.refresh_cli();
                }

                // Clear status message after ~5s
                if app.tick_count % 20 == 0 {
                    app.clear_status();
                }
            }
            None => break,
        }
    }

    Ok(())
}
