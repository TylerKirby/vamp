use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, scroll_offset};
use crate::state::TreeEntry;
use crate::theme;

/// A visible row in the tree, computed from flat entries + collapsed state
struct TreeRow<'a> {
    entry: &'a TreeEntry,
    depth: usize,
    is_expanded: bool, // only meaningful for dirs
}

/// Git status color (reuses pattern from charts.rs)
fn status_color(status: &str) -> ratatui::style::Color {
    match status {
        "M" => theme::YELLOW,
        "A" => theme::GREEN,
        "D" => theme::RED,
        "?" | "??" => theme::DIM,
        _ => theme::FG,
    }
}

/// Compute depth from path component count (e.g. "src/api/routes.rs" = depth 2)
fn path_depth(path: &str) -> usize {
    path.matches('/').count()
}

/// Get the display name (last component) from a path
fn display_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Build visible rows from flat entries considering collapsed state
fn build_visible_rows<'a>(entries: &'a [TreeEntry], collapsed: &std::collections::HashSet<String>) -> Vec<TreeRow<'a>> {
    let mut rows = Vec::new();

    for entry in entries {
        let depth = path_depth(&entry.path);

        // Check if any ancestor directory is collapsed
        let mut hidden = false;
        let parts: Vec<&str> = entry.path.split('/').collect();
        for i in 1..parts.len() {
            let ancestor = parts[..i].join("/");
            if collapsed.contains(&ancestor) {
                hidden = true;
                break;
            }
        }
        if hidden {
            continue;
        }

        let is_expanded = if entry.is_dir {
            !collapsed.contains(&entry.path)
        } else {
            false
        };

        rows.push(TreeRow {
            entry,
            depth,
            is_expanded,
        });
    }

    rows
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let entries = &app.tree_state.entries;

    if entries.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  project files",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  no files tracked",
                Style::default().fg(theme::DIM),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  waiting for data...",
                Style::default().fg(theme::DIM),
            )),
        ])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(placeholder, area);
        return;
    }

    let rows = build_visible_rows(entries, &app.collapsed_dirs);
    let mut items: Vec<ListItem> = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        let indent = "  ".repeat(row.depth);
        let name = display_name(&row.entry.path);
        let is_selected = i == app.selected_index;

        let bg = if is_selected { theme::BG_SEL } else { theme::BG };

        if row.entry.is_dir {
            // Directory row with expand/collapse indicator
            let arrow = if row.is_expanded { "\u{25be}" } else { "\u{25b8}" };
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}{} ", indent, arrow),
                    Style::default().fg(theme::BRASS).bg(bg),
                ),
                Span::styled(
                    format!("{}/", name),
                    Style::default()
                        .fg(theme::BRIGHT)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
            ])));
        } else {
            // File row with git status indicator
            let status = &row.entry.status;
            let name_style = if is_selected {
                Style::default().fg(theme::BRIGHT).bg(bg)
            } else if !status.is_empty() {
                Style::default().fg(status_color(status)).bg(bg)
            } else {
                Style::default().fg(theme::FG).bg(bg)
            };

            let mut spans = vec![
                Span::styled(format!("  {}  ", indent), Style::default().bg(bg)),
                Span::styled(name, name_style),
            ];

            // Status indicator on the right
            if !status.is_empty() {
                spans.push(Span::styled(
                    format!(" {}", status),
                    Style::default().fg(status_color(status)).bg(bg),
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

/// Count files and dirs in visible rows (for footer)
pub fn count_entries(entries: &[TreeEntry]) -> (usize, usize) {
    let files = entries.iter().filter(|e| !e.is_dir).count();
    let dirs = entries.iter().filter(|e| e.is_dir).count();
    (files, dirs)
}

/// Get the path of the entry at the given visible index (for toggle)
pub fn get_visible_entry_path(entries: &[TreeEntry], collapsed: &std::collections::HashSet<String>, index: usize) -> Option<(String, bool)> {
    let rows = build_visible_rows(entries, collapsed);
    rows.get(index).map(|r| (r.entry.path.clone(), r.entry.is_dir))
}
