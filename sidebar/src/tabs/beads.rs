use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::theme;

/// Color for issue status
fn status_color(status: &str) -> ratatui::style::Color {
    match status {
        "open" => theme::GREEN,
        "in_progress" => theme::YELLOW,
        "blocked" => theme::RED,
        "closed" | "done" => theme::DIM,
        _ => theme::FG,
    }
}

/// Icon for issue status
fn status_icon(status: &str) -> &'static str {
    match status {
        "open" => "\u{25b6}",       // ▶
        "in_progress" => "\u{25c9}", // ◉
        "blocked" => "\u{2298}",     // ⊘
        "closed" | "done" => "\u{2713}", // ✓
        _ => "?",
    }
}

/// Priority indicator
fn priority_indicator(priority: u8) -> Span<'static> {
    match priority {
        0 => Span::styled("P0", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
        1 => Span::styled("P1", Style::default().fg(theme::YELLOW)),
        2 => Span::styled("P2", Style::default().fg(theme::FG)),
        3 => Span::styled("P3", Style::default().fg(theme::DIM)),
        _ => Span::styled("P4", Style::default().fg(theme::DIM)),
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let issues = &app.beads_state.issues;

    if issues.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  issue tracker",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  no issues loaded",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  [t] toggle filter  [a] add",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    // Filter issues based on active filter
    let filtered: Vec<_> = if app.beads_filter_active {
        issues
            .iter()
            .filter(|i| i.status != "closed" && i.status != "done")
            .collect()
    } else {
        issues.iter().collect()
    };

    let mut items: Vec<ListItem> = Vec::new();

    // Header
    let filter_label = if app.beads_filter_active {
        "active"
    } else {
        "all"
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!("  {} issues", filtered.len()),
            Style::default().fg(theme::FG),
        ),
        Span::styled(
            format!("  [{}]", filter_label),
            Style::default().fg(theme::DIM),
        ),
    ])));
    items.push(ListItem::new(Line::from("")));

    for (i, issue) in filtered.iter().enumerate() {
        let icon = status_icon(&issue.status);
        let color = status_color(&issue.status);

        let is_selected = i == app.selected_index;
        let title_style = if is_selected {
            Style::default().fg(theme::BRIGHT).bg(theme::BG_SEL)
        } else {
            Style::default().fg(theme::FG)
        };

        let assignee = if issue.assignee.is_empty() {
            String::new()
        } else {
            format!(" @{}", issue.assignee)
        };

        let id_short = if issue.id.len() > 10 {
            &issue.id[..10]
        } else {
            &issue.id
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {} ", icon), Style::default().fg(color)),
            priority_indicator(issue.priority),
            Span::raw(" "),
            Span::styled(id_short, Style::default().fg(theme::DIM)),
            Span::raw(" "),
            Span::styled(&issue.title, title_style),
            Span::styled(assignee, Style::default().fg(theme::KEYS)),
        ])));
    }

    let list = List::new(items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
