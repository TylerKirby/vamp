mod app;
mod commands;
mod state;
mod tabs;
mod theme;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{App, Tab};

fn main() -> io::Result<()> {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let state_file = parse_state_file(&args);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(state_file);
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn parse_state_file(args: &[String]) -> Option<PathBuf> {
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--state-file" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut last_state_check = Instant::now();
    let state_poll_interval = Duration::from_millis(500);
    let mut last_mtime: Option<std::time::SystemTime> = None;

    // Initial state load
    app.reload_state();

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Poll for state file changes (every 500ms)
        if last_state_check.elapsed() >= state_poll_interval {
            last_state_check = Instant::now();

            if let Some(ref path) = app.state_file {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        if last_mtime.as_ref() != Some(&mtime) {
                            last_mtime = Some(mtime);
                            app.reload_state();
                        }
                    }
                }
            }

            // Also reload auxiliary data files
            app.reload_aux_data();
        }

        // Poll for keyboard events (100ms timeout for responsive UI)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit
                    KeyCode::Char('q') => {
                        app.running = false;
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.running = false;
                        return Ok(());
                    }

                    // Tab navigation
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('1') => app.select_tab(Tab::Players),
                    KeyCode::Char('2') => app.select_tab(Tab::Charts),
                    KeyCode::Char('3') => app.select_tab(Tab::Beads),
                    KeyCode::Char('4') => app.select_tab(Tab::Setlist),

                    // List navigation
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.select_prev(),

                    // Agent controls (Players tab)
                    KeyCode::Char('a') if app.active_tab == Tab::Players => {
                        if let Some(ref path) = app.state_file {
                            let cmd_file = path.with_file_name("commands.json");
                            let _ = commands::send_command(
                                &cmd_file,
                                commands::Command::create("claude", ""),
                            );
                        }
                    }
                    KeyCode::Char('f') if app.active_tab == Tab::Players => {
                        if let Some(agent) = get_selected_agent(app) {
                            if let Some(ref path) = app.state_file {
                                let cmd_file = path.with_file_name("commands.json");
                                let _ = commands::send_command(
                                    &cmd_file,
                                    commands::Command::focus(&agent),
                                );
                            }
                        }
                    }
                    KeyCode::Char('x') if app.active_tab == Tab::Players => {
                        if let Some(agent) = get_selected_agent(app) {
                            if let Some(ref path) = app.state_file {
                                let cmd_file = path.with_file_name("commands.json");
                                let _ = commands::send_command(
                                    &cmd_file,
                                    commands::Command::kill(&agent),
                                );
                            }
                        }
                    }
                    KeyCode::Char('p') if app.active_tab == Tab::Players => {
                        if let Some(agent) = get_selected_agent(app) {
                            if let Some(ref path) = app.state_file {
                                let cmd_file = path.with_file_name("commands.json");
                                let _ = commands::send_command(
                                    &cmd_file,
                                    commands::Command::pause(&agent),
                                );
                            }
                        }
                    }
                    KeyCode::Char('r') if app.active_tab == Tab::Players => {
                        if let Some(agent) = get_selected_agent(app) {
                            if let Some(ref path) = app.state_file {
                                let cmd_file = path.with_file_name("commands.json");
                                let _ = commands::send_command(
                                    &cmd_file,
                                    commands::Command::restart(&agent),
                                );
                            }
                        }
                    }

                    // Beads tab controls
                    KeyCode::Char('t') if app.active_tab == Tab::Beads => {
                        app.toggle_beads_filter();
                    }

                    _ => {}
                }
            }
        }
    }
}

/// Get the agent ID at the current selection index
fn get_selected_agent(app: &App) -> Option<String> {
    let agents = &app.state.agents;
    if agents.is_empty() {
        return None;
    }

    // Map selected_index to an agent (accounting for group headers and spacers)
    // Simple approach: just index directly into agents list
    agents.get(app.selected_index).map(|a| a.id.clone())
}
