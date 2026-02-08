use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, scroll_offset};
use crate::theme;

/// Color for git status indicator
fn status_color(status: &str) -> ratatui::style::Color {
    match status {
        "M" => theme::YELLOW,
        "A" => theme::GREEN,
        "D" => theme::RED,
        "R" => theme::CYAN,
        "?" | "??" => theme::DIM,
        _ => theme::FG,
    }
}

/// Color for agent name (based on type)
fn agent_color_by_type(agent_type: &str) -> ratatui::style::Color {
    match agent_type {
        "claude" => theme::BRASS,
        "codex" => theme::KEYS,
        "cursor" => theme::GREEN,
        _ => theme::FG,
    }
}

/// Color for agent name (based on name prefix)
fn agent_color(agent: &str) -> ratatui::style::Color {
    if agent.starts_with("claude") || agent == "main" {
        theme::BRASS
    } else if agent.starts_with("codex") {
        theme::KEYS
    } else if agent.starts_with("cursor") {
        theme::GREEN
    } else {
        theme::FG
    }
}

/// Format ahead/behind counts as compact string
fn ahead_behind_str(ahead: u32, behind: u32) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    if ahead > 0 {
        spans.push(Span::styled(
            format!("+{}", ahead),
            Style::default().fg(theme::GREEN),
        ));
    }
    if behind > 0 {
        if ahead > 0 {
            spans.push(Span::styled(" ", Style::default()));
        }
        spans.push(Span::styled(
            format!("-{}", behind),
            Style::default().fg(theme::RED),
        ));
    }
    if ahead == 0 && behind == 0 {
        spans.push(Span::styled(
            "\u{2261}",  // ≡ (in sync)
            Style::default().fg(theme::DIM),
        ));
    }
    spans
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let branches = &app.files_state.branches;
    let remotes = &app.files_state.remotes;
    let files = &app.files_state.files;
    let worktrees = &app.files_state.worktrees;

    let has_data = !branches.is_empty() || !files.is_empty() || !worktrees.is_empty();

    if !has_data {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  git overview",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  waiting for data...",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  [m] merge all",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();

    // BRANCHES section
    if !branches.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  BRANCHES",
            Style::default().fg(theme::KEYS).add_modifier(Modifier::BOLD),
        ))));

        for branch in branches {
            let current_marker = if branch.is_current {
                Span::styled("\u{2605} ", Style::default().fg(theme::BRASS)) // ★
            } else {
                Span::styled("  ", Style::default())
            };

            let name_color = if branch.is_agent {
                theme::SAX
            } else if branch.is_current {
                theme::BRIGHT
            } else {
                theme::FG
            };

            let mut spans = vec![
                Span::styled("  ", Style::default()),
                current_marker,
                Span::styled(&branch.name, Style::default().fg(name_color)),
                Span::styled("  ", Style::default()),
            ];

            // Ahead/behind counts
            spans.extend(ahead_behind_str(branch.ahead, branch.behind));

            // Tracking remote (abbreviated)
            if !branch.tracking.is_empty() {
                spans.push(Span::styled(
                    format!("  \u{2192}{}", branch.tracking.strip_prefix("origin/").unwrap_or(&branch.tracking)),
                    Style::default().fg(theme::DIM),
                ));
            }

            items.push(ListItem::new(Line::from(spans)));

            // Last commit on second line (indented)
            if !branch.last_commit.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("      ", Style::default()),
                    Span::styled(&branch.last_commit, Style::default().fg(theme::DIM)),
                ])));
            }
        }

        items.push(ListItem::new(Line::from("")));
    }

    // REMOTES section (compact — just names)
    if !remotes.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  REMOTES",
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
        ))));

        for remote in remotes {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(remote.as_str(), Style::default().fg(theme::DIM)),
            ])));
        }

        items.push(ListItem::new(Line::from("")));
    }

    // WORKTREES section
    if !worktrees.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  WORKTREES",
            Style::default().fg(theme::BRASS).add_modifier(Modifier::BOLD),
        ))));

        for wt in worktrees {
            let color = agent_color_by_type(&wt.agent_type);
            let dirty_span = if wt.dirty {
                Span::styled(" *", Style::default().fg(theme::YELLOW))
            } else {
                Span::styled(" \u{2713}", Style::default().fg(theme::GREEN))
            };
            let ahead_span = if wt.commits_ahead > 0 {
                Span::styled(
                    format!(" +{}", wt.commits_ahead),
                    Style::default().fg(theme::KEYS),
                )
            } else {
                Span::styled(" +0", Style::default().fg(theme::DIM))
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  {} ", wt.agent_id), Style::default().fg(color)),
                Span::styled(&wt.branch, Style::default().fg(theme::DIM)),
                ahead_span,
                dirty_span,
            ])));
        }

        items.push(ListItem::new(Line::from("")));
    }

    // FILES section (agent file changes)
    if !files.is_empty() {
        let conflict_count = files.iter().filter(|f| f.conflict).count();

        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("  FILES  {}", files.len()),
                Style::default().fg(theme::FG),
            ),
            if conflict_count > 0 {
                Span::styled(
                    format!("  {}!", conflict_count),
                    Style::default()
                        .fg(theme::RED)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ])));

        for (i, file) in files.iter().enumerate() {
            let conflict_marker = if file.conflict {
                Span::styled(
                    " !",
                    Style::default()
                        .fg(theme::RED)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            };

            let agent_spans: Vec<Span> = file
                .agents
                .iter()
                .map(|(agent, status)| {
                    Span::styled(
                        format!(" {}:{}", agent, status),
                        Style::default().fg(agent_color(agent)),
                    )
                })
                .collect();

            let primary_status = file
                .agents
                .values()
                .next()
                .map(|s| s.as_str())
                .unwrap_or("?");

            let is_selected = i == app.selected_index;
            let path_style = if is_selected {
                Style::default().fg(theme::BRIGHT).bg(theme::BG_SEL)
            } else {
                Style::default().fg(status_color(primary_status))
            };

            let mut spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(&file.path, path_style),
                conflict_marker,
            ];
            spans.extend(agent_spans);

            if let Some(locker) = app.files_state.locks.get(&file.path) {
                spans.push(Span::styled(
                    format!(" [{}]", locker),
                    Style::default().fg(theme::MAGENTA),
                ));
            }

            items.push(ListItem::new(Line::from(spans)));
        }
    }

    // Scroll: keep selected item visible
    let visible = area.height as usize;
    let offset = scroll_offset(app.selected_index, visible);
    let visible_items: Vec<ListItem> = items.into_iter().skip(offset).collect();

    let list = List::new(visible_items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
