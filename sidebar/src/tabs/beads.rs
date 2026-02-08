use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, scroll_offset};
use crate::theme;

/// Color for issue status (pub so ui.rs can reuse for detail popup)
pub fn status_color(status: &str) -> ratatui::style::Color {
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
                "  [t] toggle  [s] status  [l] type",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    // Combined filtering: active toggle + status filter + type filter
    let filtered: Vec<_> = issues
        .iter()
        .filter(|i| {
            // Active/all toggle
            if app.beads_filter_active && (i.status == "closed" || i.status == "done") {
                return false;
            }
            // Status filter (use effective_status to account for dependency-based blocking)
            if let Some(ref sf) = app.beads_status_filter {
                if i.effective_status() != *sf {
                    return false;
                }
            }
            // Type filter
            if let Some(ref tf) = app.beads_type_filter {
                if i.issue_type != *tf {
                    return false;
                }
            }
            true
        })
        .collect();

    let mut items: Vec<ListItem> = Vec::new();

    // Header with filter labels
    let active_label = if app.beads_filter_active { "active" } else { "all" };
    let status_label = app.beads_status_filter.as_deref().unwrap_or("*");
    let type_label = app.beads_type_filter.as_deref().unwrap_or("*");

    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!("  {} issues", filtered.len()),
            Style::default().fg(theme::FG),
        ),
        Span::styled(
            format!("  [{}]", active_label),
            Style::default().fg(theme::DIM),
        ),
        Span::styled(
            format!(" s:{}", status_label),
            Style::default().fg(theme::DIM),
        ),
        Span::styled(
            format!(" l:{}", type_label),
            Style::default().fg(theme::DIM),
        ),
    ])));
    items.push(ListItem::new(Line::from("")));

    // header_rows = 2 (header line + blank line), so issue i maps to items[i + header_rows]
    let header_rows = items.len();

    for (i, issue) in filtered.iter().enumerate() {
        let effective = issue.effective_status();
        let icon = status_icon(effective);
        let color = status_color(effective);

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

        // Show more of issue ID: 16 chars with .. suffix
        let id_short: String = if issue.id.len() > 16 {
            let mut s: String = issue.id.chars().take(16).collect();
            s.push_str("..");
            s
        } else {
            issue.id.clone()
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

    // Scroll: keep selected item visible
    let visible = area.height as usize;
    let selected_row = app.selected_index + header_rows;
    let offset = scroll_offset(selected_row, visible);
    let visible_items: Vec<ListItem> = items.into_iter().skip(offset).collect();

    let list = List::new(visible_items).block(Block::default().borders(Borders::NONE));
    frame.render_widget(list, area);
}
