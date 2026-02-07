use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::theme;

/// Status icon for agent state
fn status_icon(status: &str) -> &'static str {
    match status {
        "running" => "\u{2669}",  // ♩
        "waiting" => "\u{1d13e}", // 𝄾
        "paused" => "\u{25cb}",   // ○
        "error" => "\u{2717}",    // ✗
        "stopped" => "\u{25cb}",  // ○
        _ => "?",
    }
}

/// Color for agent type group
fn type_color(agent_type: &str) -> ratatui::style::Color {
    match agent_type {
        "claude" => theme::BRASS,
        "codex" => theme::KEYS,
        "cursor" => theme::GREEN,
        _ => theme::FG,
    }
}

/// Group label for agent type
fn type_label(agent_type: &str) -> &'static str {
    match agent_type {
        "claude" => "BRASS",
        "codex" => "KEYS",
        "cursor" => "STRINGS",
        _ => "OTHER",
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let agents = &app.state.agents;

    if agents.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No agents running",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  press [a] to add an agent",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::NONE),
        );
        frame.render_widget(placeholder, area);
        return;
    }

    // Group agents by type
    let mut grouped: Vec<(&str, Vec<usize>)> = Vec::new();
    let type_order = ["claude", "codex", "cursor"];

    for t in &type_order {
        let indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.agent_type == *t)
            .map(|(i, _)| i)
            .collect();
        if !indices.is_empty() {
            grouped.push((t, indices));
        }
    }

    // Collect agents with unknown types
    let known: Vec<&str> = type_order.to_vec();
    let other_indices: Vec<usize> = agents
        .iter()
        .enumerate()
        .filter(|(_, a)| !known.contains(&a.agent_type.as_str()))
        .map(|(i, _)| i)
        .collect();
    if !other_indices.is_empty() {
        grouped.push(("other", other_indices));
    }

    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_index = 0;

    for (agent_type, indices) in &grouped {
        // Group header
        let color = type_color(agent_type);
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {} {}", "\u{25b8}", type_label(agent_type)),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))));
        flat_index += 1;

        for &idx in indices {
            let agent = &agents[idx];
            let icon = status_icon(&agent.status);
            let status_color = match agent.status.as_str() {
                "running" => theme::GREEN,
                "waiting" => theme::YELLOW,
                "paused" => theme::DIM,
                "error" => theme::RED,
                _ => theme::DIM,
            };

            let task_str = if agent.task.is_empty() {
                String::new()
            } else {
                format!(" {}", agent.task)
            };

            let style = if flat_index == app.selected_index + 1 {
                Style::default().fg(theme::BRIGHT).bg(theme::BG_SEL)
            } else {
                Style::default().fg(theme::FG)
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("    {} ", icon), Style::default().fg(status_color)),
                Span::styled(&agent.name, style),
                Span::styled(task_str, Style::default().fg(theme::DIM)),
            ])));
            flat_index += 1;
        }

        // Spacer after group
        items.push(ListItem::new(Line::from("")));
        flat_index += 1;
    }

    let list = List::new(items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
