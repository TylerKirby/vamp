use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::theme;

/// Color for event type
fn event_color(event: &str) -> ratatui::style::Color {
    match event {
        "agent_create" | "create" => theme::GREEN,
        "agent_kill" | "kill" => theme::RED,
        "agent_pause" | "pause" => theme::YELLOW,
        "agent_resume" | "resume" => theme::GREEN,
        "commit" | "git_commit" => theme::KEYS,
        "bead_close" | "bead_create" => theme::CYAN,
        "error" => theme::RED,
        _ => theme::FG,
    }
}

/// Agent name color
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

/// Format timestamp for display (show HH:MM:SS from ISO timestamp)
fn format_time(timestamp: &str) -> &str {
    // ISO format: 2026-02-07T10:30:00Z — extract HH:MM:SS
    timestamp.get(11..19).unwrap_or(timestamp)
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let log = &app.activity_log;

    if log.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  activity log",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  no activity yet",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  events appear as agents work",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();

    // Show most recent entries first (reversed), limited to visible area
    let visible_height = area.height.saturating_sub(2) as usize;
    let start = if log.len() > visible_height {
        log.len() - visible_height
    } else {
        0
    };

    for entry in &log[start..] {
        let time = format_time(&entry.timestamp);
        let color = event_color(&entry.event);

        let detail = if entry.detail.is_empty() {
            String::new()
        } else if entry.detail.chars().count() > 30 {
            let truncated: String = entry.detail.chars().take(28).collect();
            format!(" {}..", truncated)
        } else {
            format!(" {}", entry.detail)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("  {} ", time),
                Style::default().fg(theme::DIM),
            ),
            Span::styled(
                format!("{} ", entry.agent),
                Style::default().fg(agent_color(&entry.agent)),
            ),
            Span::styled(&entry.event, Style::default().fg(color)),
            Span::styled(detail, Style::default().fg(theme::DIM)),
        ])));
    }

    let list = List::new(items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
