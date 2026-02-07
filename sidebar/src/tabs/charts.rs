use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
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

/// Color for agent name (based on type prefix)
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

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let files = &app.files_state.files;

    if files.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  file changes across agents",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  waiting for agent activity...",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  [d] diff  [l] lock  [u] unlock",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    // Count stats
    let modified_count = files.iter().filter(|f| !f.conflict).count();
    let conflict_count = files.iter().filter(|f| f.conflict).count();

    let mut items: Vec<ListItem> = Vec::new();

    // Header with counts
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!("  {} files", files.len()),
            Style::default().fg(theme::FG),
        ),
        Span::styled(
            format!("  {}M", modified_count),
            Style::default().fg(theme::YELLOW),
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
    items.push(ListItem::new(Line::from("")));

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

        // Build agent attribution spans
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

        // Get primary status for color
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

        // Show lock info
        if let Some(locker) = app.files_state.locks.get(&file.path) {
            spans.push(Span::styled(
                format!(" [{}]", locker),
                Style::default().fg(theme::MAGENTA),
            ));
        }

        items.push(ListItem::new(Line::from(spans)));
    }

    let list = List::new(items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
