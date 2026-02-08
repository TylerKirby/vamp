use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, Dialog, Tab, AGENT_TYPES};
use crate::tabs;
use crate::theme;

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Background
    let bg_block = Block::default().style(Style::default().bg(theme::BG));
    frame.render_widget(bg_block, size);

    // Layout: header (1) + tab bar (1) + content (flex) + footer (1)
    let chunks = Layout::vertical([
        Constraint::Length(2), // header
        Constraint::Length(1), // tab bar
        Constraint::Min(3),   // content
        Constraint::Length(1), // footer
    ])
    .split(size);

    render_header(frame, chunks[0]);
    render_tab_bar(frame, chunks[1], app);
    render_content(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
    render_dialog(frame, size, app);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " \u{266b} ",
            Style::default()
                .fg(theme::BRASS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "vamp sessions",
            Style::default()
                .fg(theme::BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::BORDER))
            .style(Style::default().bg(theme::BG_ALT)),
    );
    frame.render_widget(header, area);
}

fn render_tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let use_short = area.width < 55;
    let tabs: Vec<Span> = Tab::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let label = if use_short { tab.short_label() } else { tab.label() };
            let num = format!("{}", i + 1);

            let style = if *tab == app.active_tab {
                Style::default()
                    .fg(theme::BRIGHT)
                    .bg(theme::BG_SEL)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM)
            };

            let num_style = if *tab == app.active_tab {
                Style::default()
                    .fg(theme::BRASS)
                    .bg(theme::BG_SEL)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM)
            };

            let mut spans = vec![
                Span::styled(format!(" {}", num), num_style),
                Span::styled(format!(":{} ", label), style),
            ];

            if i < Tab::ALL.len() - 1 {
                spans.push(Span::styled(
                    "\u{2502}",
                    Style::default().fg(theme::BORDER),
                ));
            }

            spans
        })
        .collect();

    let tab_bar =
        Paragraph::new(Line::from(tabs)).style(Style::default().bg(theme::BG_ALT).fg(theme::DIM));
    frame.render_widget(tab_bar, area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &mut App) {
    app.content_area = Some(area);
    match app.active_tab {
        Tab::Players => tabs::players::render(frame, area, app),
        Tab::Charts => tabs::charts::render(frame, area, app),
        Tab::Beads => tabs::beads::render(frame, area, app),
        Tab::Setlist => tabs::setlist::render(frame, area, app),
        Tab::Files => tabs::files::render(frame, area, app),
        Tab::Metrics => tabs::metrics::render(frame, area, app),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &mut App) {
    // Show status message if active (overrides normal footer)
    // Take the message out to avoid borrow issues, put it back if still valid
    if let Some((msg, expiry)) = app.status_message.take() {
        if std::time::Instant::now() < expiry {
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(&msg, Style::default().fg(theme::BRASS)),
            ]))
            .style(Style::default().bg(theme::BG_ALT));
            frame.render_widget(footer, area);
            app.status_message = Some((msg, expiry));
            return;
        }
        // expired — leave it as None
    }

    let footer_text = match app.active_tab {
        Tab::Players => {
            let agents = &app.state.agents;
            let totals = &app.state.totals;
            let running = agents.iter().filter(|a| a.status == "running").count();
            let waiting = agents.iter().filter(|a| a.status == "waiting").count();
            let idle = agents.iter().filter(|a| a.status != "running" && a.status != "waiting" && a.status != "error").count();
            let tok_str = if totals.tokens > 1000 { format!("{}K", totals.tokens / 1000) } else { format!("{}", totals.tokens) };
            if agents.is_empty() {
                " \u{2669}0  [A]dd  [q]uit".to_string()
            } else {
                format!(" \u{2669}{}  \u{1d13e}{}  \u{25cb}{}  [A]dd [X]kill [R]ename  tok:{} ${:.2}", running, waiting, idle, tok_str, totals.cost_usd)
            }
        }
        Tab::Charts => {
            let br = app.files_state.branches.len();
            let wt = app.files_state.worktrees.len();
            let conflicts = app.files_state.files.iter().filter(|f| f.conflict).count();
            if conflicts > 0 {
                format!(" {} br  {} wt  {}! conflicts  [m]erge", br, wt, conflicts)
            } else {
                format!(" {} branches  {} worktrees  [m]erge", br, wt)
            }
        }
        Tab::Beads => {
            let issues = &app.beads_state.issues;
            let open = issues.iter().filter(|i| i.status == "open").count();
            let blocked = issues.iter().filter(|i| i.status == "blocked").count();
            let status_label = app.beads_status_filter.as_deref().unwrap_or("all");
            let type_label = app.beads_type_filter.as_deref().unwrap_or("all");
            format!(" {} open  {} blocked  [t]oggle [s]:{} [l]:{} [Enter]detail", open, blocked, status_label, type_label)
        }
        Tab::Setlist => {
            let total = app.activity_log.len();
            format!(" {} events", total)
        }
        Tab::Files => {
            let (files, dirs) = tabs::files::count_entries(&app.tree_state.entries);
            format!(" {} files  {} dirs  [Enter]toggle", files, dirs)
        }
        Tab::Metrics => {
            let sys = &app.metrics_state.system;
            let msgs = app.metrics_state.claude.today_messages;
            format!(
                " load:{:.2}  mem:{}%  msgs:{}",
                sys.load_avg[0],
                sys.memory_pct as u64,
                msgs
            )
        }
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(theme::DIM),
    )))
    .style(Style::default().bg(theme::BG_ALT));
    frame.render_widget(footer, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn render_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let dialog = match &app.dialog {
        Some(d) => d,
        None => return,
    };

    match dialog {
        Dialog::AddAgent { selected } => {
            let popup = centered_rect(22, 9, area);
            frame.render_widget(Clear, popup);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(Span::styled(
                    " add agent ",
                    Style::default().fg(theme::BRASS).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme::BG_ALT));
            frame.render_widget(block, popup);

            let inner = Rect::new(popup.x + 2, popup.y + 2, popup.width.saturating_sub(4), popup.height.saturating_sub(4));

            let mut lines = Vec::new();
            for (i, agent_type) in AGENT_TYPES.iter().enumerate() {
                if i == *selected {
                    lines.push(Line::from(vec![
                        Span::styled("\u{25b8} ", Style::default().fg(theme::BRASS)),
                        Span::styled(
                            *agent_type,
                            Style::default().fg(theme::BRIGHT).add_modifier(Modifier::BOLD),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(*agent_type, Style::default().fg(theme::FG)),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "enter \u{00b7} esc",
                Style::default().fg(theme::DIM),
            )));

            let content = Paragraph::new(lines);
            frame.render_widget(content, inner);
        }
        Dialog::ConfirmKill { agent_name, .. } => {
            let popup = centered_rect(24, 6, area);
            frame.render_widget(Clear, popup);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(Span::styled(
                    " kill agent ",
                    Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme::BG_ALT));
            frame.render_widget(block, popup);

            let inner = Rect::new(popup.x + 2, popup.y + 2, popup.width.saturating_sub(4), popup.height.saturating_sub(4));

            let lines = vec![
                Line::from(vec![
                    Span::styled("kill ", Style::default().fg(theme::FG)),
                    Span::styled(agent_name.as_str(), Style::default().fg(theme::BRIGHT)),
                    Span::styled("?", Style::default().fg(theme::FG)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[y]", Style::default().fg(theme::RED)),
                    Span::styled("es \u{00b7} ", Style::default().fg(theme::DIM)),
                    Span::styled("[n]", Style::default().fg(theme::DIM)),
                    Span::styled("o", Style::default().fg(theme::DIM)),
                ]),
            ];

            let content = Paragraph::new(lines);
            frame.render_widget(content, inner);
        }
        Dialog::RenameAgent { input, .. } => {
            let popup = centered_rect(30, 6, area);
            frame.render_widget(Clear, popup);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER))
                .title(Span::styled(
                    " rename agent ",
                    Style::default().fg(theme::BRASS).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme::BG_ALT));
            frame.render_widget(block, popup);

            let inner = Rect::new(popup.x + 2, popup.y + 2, popup.width.saturating_sub(4), popup.height.saturating_sub(4));

            let lines = vec![
                Line::from(vec![
                    Span::styled(input.as_str(), Style::default().fg(theme::BRIGHT)),
                    Span::styled("_", Style::default().fg(theme::BRASS)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "enter \u{00b7} esc",
                    Style::default().fg(theme::DIM),
                )),
            ];

            let content = Paragraph::new(lines);
            frame.render_widget(content, inner);
        }
        Dialog::BeadDetail { issue_index, scroll } => {
            // Get the filtered issue at the given index
            let issues = &app.beads_state.issues;
            let filtered: Vec<_> = issues
                .iter()
                .filter(|i| {
                    if app.beads_filter_active && (i.status == "closed" || i.status == "done") {
                        return false;
                    }
                    if let Some(ref sf) = app.beads_status_filter {
                        if i.status != *sf { return false; }
                    }
                    if let Some(ref tf) = app.beads_type_filter {
                        if i.issue_type != *tf { return false; }
                    }
                    true
                })
                .collect();

            if let Some(issue) = filtered.get(*issue_index) {
                // Use most of the screen for the detail popup
                let popup_w = area.width.saturating_sub(4).min(50);
                let popup_h = area.height.saturating_sub(4).min(30);
                let popup = centered_rect(popup_w, popup_h, area);
                frame.render_widget(Clear, popup);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::BORDER))
                    .title(Span::styled(
                        " issue detail ",
                        Style::default().fg(theme::KEYS).add_modifier(Modifier::BOLD),
                    ))
                    .style(Style::default().bg(theme::BG_ALT));
                frame.render_widget(block, popup);

                let inner = Rect::new(popup.x + 1, popup.y + 1, popup.width.saturating_sub(2), popup.height.saturating_sub(2));
                let max_width = inner.width.saturating_sub(2) as usize;

                let color = tabs::beads::status_color(&issue.status);

                let assignee = if issue.assignee.is_empty() {
                    "unassigned".to_string()
                } else {
                    format!("@{}", issue.assignee)
                };

                // Format timestamp: show date portion from ISO
                let fmt_date = |ts: &str| -> String {
                    ts.get(..10).map(|s| s.to_string()).unwrap_or_else(|| if ts.is_empty() { "-".to_string() } else { ts.to_string() })
                };

                let mut lines: Vec<Line> = Vec::new();

                // Title
                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(&issue.title, Style::default().fg(theme::BRIGHT).add_modifier(Modifier::BOLD)),
                ]));

                // ID
                lines.push(Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(&issue.id, Style::default().fg(theme::DIM)),
                ]));

                lines.push(Line::from(""));

                // Status + priority + type + assignee
                lines.push(Line::from(vec![
                    Span::styled(" status   ", Style::default().fg(theme::DIM)),
                    Span::styled(&issue.status, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" priority ", Style::default().fg(theme::DIM)),
                    Span::styled(format!("P{}", issue.priority), Style::default().fg(
                        match issue.priority { 0 => theme::RED, 1 => theme::YELLOW, _ => theme::FG }
                    )),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" type     ", Style::default().fg(theme::DIM)),
                    Span::styled(&issue.issue_type, Style::default().fg(theme::FG)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(" assignee ", Style::default().fg(theme::DIM)),
                    Span::styled(&assignee, Style::default().fg(theme::KEYS)),
                ]));

                // Dependencies
                if issue.dependency_count > 0 || issue.dependent_count > 0 {
                    lines.push(Line::from(""));
                    if issue.dependency_count > 0 {
                        lines.push(Line::from(vec![
                            Span::styled(" blocked by ", Style::default().fg(theme::DIM)),
                            Span::styled(format!("{} issue(s)", issue.dependency_count), Style::default().fg(theme::RED)),
                        ]));
                    }
                    if issue.dependent_count > 0 {
                        lines.push(Line::from(vec![
                            Span::styled(" blocks     ", Style::default().fg(theme::DIM)),
                            Span::styled(format!("{} issue(s)", issue.dependent_count), Style::default().fg(theme::YELLOW)),
                        ]));
                    }
                }

                // Labels
                if !issue.labels.is_empty() {
                    lines.push(Line::from(""));
                    let label_str = issue.labels.join(", ");
                    lines.push(Line::from(vec![
                        Span::styled(" labels   ", Style::default().fg(theme::DIM)),
                        Span::styled(label_str, Style::default().fg(theme::CYAN)),
                    ]));
                }

                // Timestamps
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(" created  ", Style::default().fg(theme::DIM)),
                    Span::styled(fmt_date(&issue.created_at), Style::default().fg(theme::FG)),
                    Span::styled("  updated ", Style::default().fg(theme::DIM)),
                    Span::styled(fmt_date(&issue.updated_at), Style::default().fg(theme::FG)),
                ]));
                if !issue.closed_at.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(" closed   ", Style::default().fg(theme::DIM)),
                        Span::styled(fmt_date(&issue.closed_at), Style::default().fg(theme::FG)),
                    ]));
                }

                // Close reason
                if !issue.close_reason.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(" close reason:", Style::default().fg(theme::DIM))));
                    for line in issue.close_reason.lines() {
                        let truncated: String = line.chars().take(max_width).collect();
                        lines.push(Line::from(Span::styled(format!(" {}", truncated), Style::default().fg(theme::FG))));
                    }
                }

                // Description
                lines.push(Line::from(""));
                if issue.description.is_empty() {
                    lines.push(Line::from(Span::styled(" (no description)", Style::default().fg(theme::DIM))));
                } else {
                    lines.push(Line::from(Span::styled(" description:", Style::default().fg(theme::DIM))));
                    for line in issue.description.lines() {
                        let truncated: String = line.chars().take(max_width).collect();
                        lines.push(Line::from(Span::styled(format!(" {}", truncated), Style::default().fg(theme::FG))));
                    }
                }

                // Footer hint
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " j/k scroll \u{00b7} esc/q close",
                    Style::default().fg(theme::DIM),
                )));

                // Apply scroll offset
                let visible_height = inner.height as usize;
                let max_scroll = lines.len().saturating_sub(visible_height);
                let effective_scroll = (*scroll).min(max_scroll);
                let visible_lines: Vec<Line> = lines.into_iter().skip(effective_scroll).take(visible_height).collect();

                let content = Paragraph::new(visible_lines);
                frame.render_widget(content, inner);
            }
        }
    }
}
