use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, FlatRow, build_flat_list};
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
        "claude" => "CLAUDE",
        "codex" => "CODEX",
        "cursor" => "CURSOR",
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
                "  press [A] to add an agent",
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

    let flat = build_flat_list(agents);
    let mut items: Vec<ListItem> = Vec::new();

    for (flat_index, row) in flat.iter().enumerate() {
        match row {
            FlatRow::Header(agent_type) => {
                let color = type_color(agent_type);
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("  {} {}", "\u{25b8}", type_label(agent_type)),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))));
            }
            FlatRow::Agent(idx) => {
                let agent = &agents[*idx];
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

                // Check if this agent is the focused one
                let focused = &app.state.focused_agent;
                let is_focused = focused == &agent.id
                    || (focused.is_empty() && agent.id == "main");

                let style = if flat_index == app.selected_index + 1 {
                    let mut s = Style::default().fg(theme::BRIGHT).bg(theme::BG_SEL);
                    if is_focused { s = s.add_modifier(Modifier::BOLD); }
                    s
                } else {
                    let mut s = Style::default().fg(theme::FG);
                    if is_focused { s = s.add_modifier(Modifier::BOLD); }
                    s
                };

                // Star indicator for focused agent
                let focus_indicator = if is_focused {
                    Span::styled("\u{2605} ", Style::default().fg(theme::BRASS))
                } else {
                    Span::styled("  ", Style::default())
                };

                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", icon), Style::default().fg(status_color)),
                    focus_indicator,
                    Span::styled(&agent.name, style),
                    Span::styled(task_str, Style::default().fg(theme::DIM)),
                ])));
            }
            FlatRow::Spacer => {
                items.push(ListItem::new(Line::from("")));
            }
        }
    }

    // Scroll: keep selected item visible
    let visible = area.height as usize;
    let offset = crate::app::scroll_offset(app.selected_index, visible);
    let visible_items: Vec<ListItem> = items.into_iter().skip(offset).collect();

    let list = List::new(visible_items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
