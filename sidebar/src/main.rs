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
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{App, Dialog, FlatRow, Tab, AGENT_TYPES, build_flat_list};

fn main() -> io::Result<()> {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let demo_mode = args.iter().any(|a| a == "--demo");
    let state_file = parse_state_file(&args);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(state_file);

    if demo_mode {
        let (state, files, beads, activity, tree, metrics) = state::demo_state();
        app.state = state;
        app.files_state = files;
        app.beads_state = beads;
        app.activity_log = activity;
        app.tree_state = tree;
        app.metrics_state = metrics;
        app.snap_to_first_agent();
    }

    let result = run_app(&mut terminal, &mut app, demo_mode);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App, demo_mode: bool) -> io::Result<()> {
    let mut last_state_check = Instant::now();
    let state_poll_interval = Duration::from_millis(500);
    let mut last_mtime: Option<std::time::SystemTime> = None;

    // Initial state load (skip in demo mode — data is already populated)
    if !demo_mode {
        app.reload_state();
    }

    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Poll for state file changes (every 500ms) — skip in demo mode
        if !demo_mode && last_state_check.elapsed() >= state_poll_interval {
            last_state_check = Instant::now();

            if let Some(ref path) = app.state_file {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        if last_mtime.as_ref() != Some(&mtime) {
                            last_mtime = Some(mtime);
                            let prev_count = app.state.agents.len();
                            app.reload_state();
                            // Clear status message when agent count changes
                            if app.state.agents.len() != prev_count {
                                app.status_message = None;
                            }
                        }
                    }
                }
            }

            // Also reload auxiliary data files
            app.reload_aux_data();
        }

        // Poll for events (100ms timeout for responsive UI)
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Dialog key handling — intercept all keys when dialog is open
                    if let Some(dialog) = app.dialog.take() {
                        match dialog {
                            Dialog::AddAgent { selected } => match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    let new_sel = if selected == 0 {
                                        AGENT_TYPES.len() - 1
                                    } else {
                                        selected - 1
                                    };
                                    app.dialog = Some(Dialog::AddAgent { selected: new_sel });
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let new_sel = (selected + 1) % AGENT_TYPES.len();
                                    app.dialog = Some(Dialog::AddAgent { selected: new_sel });
                                }
                                KeyCode::Enter => {
                                    let agent_type = AGENT_TYPES[selected];
                                    send_cmd(app, commands::Command::create(agent_type, ""));
                                    app.set_status(format!("spinning up {} agent...", agent_type), 10);
                                }
                                KeyCode::Esc | KeyCode::Char('q') => {}
                                _ => {
                                    app.dialog = Some(Dialog::AddAgent { selected });
                                }
                            },
                            Dialog::ConfirmKill { agent_id, agent_name } => match key.code {
                                KeyCode::Char('y') | KeyCode::Enter => {
                                    app.set_status(format!("killing {}...", agent_name), 10);
                                    send_cmd(app, commands::Command::kill(&agent_id));
                                }
                                KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {}
                                _ => {
                                    app.dialog = Some(Dialog::ConfirmKill { agent_id, agent_name });
                                }
                            },
                            Dialog::RenameAgent { agent_id, mut input } => match key.code {
                                KeyCode::Enter => {
                                    if !input.is_empty() {
                                        send_cmd(app, commands::Command::rename(&agent_id, &input));
                                    }
                                }
                                KeyCode::Esc => {}
                                KeyCode::Backspace => {
                                    input.pop();
                                    app.dialog = Some(Dialog::RenameAgent { agent_id, input });
                                }
                                KeyCode::Char(c) if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => {
                                    if input.len() < 64 {
                                        input.push(c);
                                    }
                                    app.dialog = Some(Dialog::RenameAgent { agent_id, input });
                                }
                                KeyCode::Char(_) => {
                                    // Reject invalid characters silently
                                    app.dialog = Some(Dialog::RenameAgent { agent_id, input });
                                }
                                _ => {
                                    app.dialog = Some(Dialog::RenameAgent { agent_id, input });
                                }
                            },
                            Dialog::BeadDetail { issue_index, scroll } => match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {}
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.dialog = Some(Dialog::BeadDetail { issue_index, scroll: scroll.saturating_add(1) });
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.dialog = Some(Dialog::BeadDetail { issue_index, scroll: scroll.saturating_sub(1) });
                                }
                                _ => {
                                    app.dialog = Some(Dialog::BeadDetail { issue_index, scroll });
                                }
                            },
                        }
                        continue;
                    }

                    match key.code {
                        // Quit
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(());
                        }

                        // Tab navigation
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.prev_tab(),
                        KeyCode::Char('1') => app.select_tab(Tab::Players),
                        KeyCode::Char('2') => app.select_tab(Tab::Charts),
                        KeyCode::Char('3') => app.select_tab(Tab::Beads),
                        KeyCode::Char('4') => app.select_tab(Tab::Setlist),
                        KeyCode::Char('5') => app.select_tab(Tab::Files),
                        KeyCode::Char('6') => app.select_tab(Tab::Metrics),

                        // List navigation
                        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),

                        // Agent controls (Players tab)
                        KeyCode::Char('A') if app.active_tab == Tab::Players => {
                            app.dialog = Some(Dialog::AddAgent { selected: 0 });
                        }
                        KeyCode::Char('X') if app.active_tab == Tab::Players => {
                            if let Some(agent_id) = get_selected_agent(app) {
                                let agent_name = app.state.agents.iter()
                                    .find(|a| a.id == agent_id)
                                    .map(|a| a.name.clone())
                                    .unwrap_or_else(|| agent_id.clone());
                                app.dialog = Some(Dialog::ConfirmKill { agent_id, agent_name });
                            }
                        }
                        KeyCode::Char('f') if app.active_tab == Tab::Players => {
                            if let Some(agent) = get_selected_agent(app) {
                                send_cmd(app, commands::Command::focus(&agent));
                            }
                        }
                        KeyCode::Char('p') if app.active_tab == Tab::Players => {
                            if let Some(agent) = get_selected_agent(app) {
                                send_cmd(app, commands::Command::pause(&agent));
                            }
                        }
                        KeyCode::Char('r') if app.active_tab == Tab::Players => {
                            if let Some(agent) = get_selected_agent(app) {
                                send_cmd(app, commands::Command::restart(&agent));
                            }
                        }
                        KeyCode::Char('R') if app.active_tab == Tab::Players => {
                            if let Some(agent_id) = get_selected_agent(app) {
                                let current_name = app.state.agents.iter()
                                    .find(|a| a.id == agent_id)
                                    .map(|a| a.name.clone())
                                    .unwrap_or_default();
                                app.dialog = Some(Dialog::RenameAgent { agent_id, input: current_name });
                            }
                        }

                        // Beads tab controls
                        KeyCode::Char('t') if app.active_tab == Tab::Beads => {
                            app.toggle_beads_filter();
                        }
                        KeyCode::Char('s') if app.active_tab == Tab::Beads => {
                            app.cycle_beads_status_filter();
                        }
                        KeyCode::Char('l') if app.active_tab == Tab::Beads => {
                            app.cycle_beads_type_filter();
                        }
                        KeyCode::Enter if app.active_tab == Tab::Files => {
                            if let Some((path, is_dir)) = tabs::files::get_visible_entry_path(
                                &app.tree_state.entries,
                                &app.collapsed_dirs,
                                app.selected_index,
                            ) {
                                if is_dir {
                                    if app.collapsed_dirs.contains(&path) {
                                        app.collapsed_dirs.remove(&path);
                                    } else {
                                        app.collapsed_dirs.insert(path);
                                    }
                                }
                            }
                        }
                        KeyCode::Enter if app.active_tab == Tab::Beads => {
                            app.dialog = Some(Dialog::BeadDetail { issue_index: app.selected_index, scroll: 0 });
                        }

                        // Charts tab controls
                        KeyCode::Char('m') if app.active_tab == Tab::Charts => {
                            send_cmd(app, commands::Command::merge("--all"));
                        }

                        _ => {}
                    }
                }
                Event::Mouse(mouse) => {
                    if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                        && app.active_tab == Tab::Players
                    {
                        if let Some(area) = app.content_area {
                            let screen_row = (mouse.row.saturating_sub(area.y)) as usize;
                            let flat = build_flat_list(&app.state.agents);
                            // Adjust for scroll offset
                            let visible = area.height as usize;
                            let offset = app::scroll_offset(app.selected_index, visible);
                            let flat_index = screen_row + offset;
                            if let Some(FlatRow::Agent(idx)) = flat.get(flat_index) {
                                app.selected_index = flat_index.saturating_sub(1);
                                if let Some(agent) = app.state.agents.get(*idx) {
                                    let agent_id = agent.id.clone();
                                    send_cmd(app, commands::Command::focus(&agent_id));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Send a command to the bash watcher via commands.json
fn send_cmd(app: &App, cmd: commands::Command) {
    if let Some(ref path) = app.state_file {
        let cmd_file = path.with_file_name("commands.json");
        let _ = commands::send_command(&cmd_file, cmd);
    }
}

/// Get the agent ID at the current selection index
fn get_selected_agent(app: &App) -> Option<String> {
    let agents = &app.state.agents;
    if agents.is_empty() {
        return None;
    }

    let flat = build_flat_list(agents);
    // selected_index + 1 maps to the flat_index (matching players.rs highlight logic)
    let target = app.selected_index + 1;
    match flat.get(target) {
        Some(FlatRow::Agent(idx)) => Some(agents[*idx].id.clone()),
        _ => None,
    }
}
