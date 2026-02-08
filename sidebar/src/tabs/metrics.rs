use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::theme;

/// Status icon and color based on agent status + process liveness
fn status_icon_and_color(status: &str, process_status: &str) -> (&'static str, ratatui::style::Color) {
    if process_status == "dead" {
        return ("\u{2717}", theme::RED); // ✗
    }
    match status {
        "running" => ("\u{2669}", theme::GREEN),   // ♩
        "waiting" => ("\u{1d13e}", theme::YELLOW),  // 𝄾
        "paused"  => ("\u{25cb}", theme::DIM),      // ○
        "error"   => ("\u{2717}", theme::RED),      // ✗
        "stopped" => ("\u{25a0}", theme::DIM),      // ■
        _         => ("?", theme::DIM),
    }
}

/// Color for agent type
fn agent_type_color(agent_type: &str) -> ratatui::style::Color {
    match agent_type {
        "claude" => theme::BRASS,
        "codex"  => theme::KEYS,
        "cursor" => theme::GREEN,
        _        => theme::FG,
    }
}

/// Format token count: 0-999 as-is, 1K-999K with K suffix, 1M+ with M suffix
fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        let m = n as f64 / 1_000_000.0;
        if m >= 10.0 {
            format!("{}M", m as u64)
        } else {
            format!("{:.1}M", m)
        }
    } else if n >= 1_000 {
        let k = n as f64 / 1_000.0;
        if k >= 10.0 {
            format!("{}K", k as u64)
        } else {
            format!("{:.1}K", k)
        }
    } else {
        format!("{}", n)
    }
}

/// Format number with commas: 1234 -> "1,234"
fn format_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Build sparkline from hour_counts using block chars
fn sparkline(counts: &[u64]) -> String {
    let blocks = [' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
    let max = counts.iter().copied().max().unwrap_or(1).max(1);
    counts
        .iter()
        .map(|&v| {
            let idx = ((v as f64 / max as f64) * 8.0) as usize;
            blocks[idx.min(8)]
        })
        .collect()
}

/// Format memory size in human-readable form
fn format_mem(mb: u64) -> String {
    if mb >= 1024 {
        let gb = mb as f64 / 1024.0;
        format!("{:.1}G", gb)
    } else {
        format!("{}M", mb)
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let metrics = &app.metrics_state;

    if metrics.updated_at.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  system & claude metrics",
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

    let sys = &metrics.system;
    let claude = &metrics.claude;
    let agents = &metrics.agents;
    let mut lines: Vec<Line> = Vec::new();

    // SYSTEM section
    lines.push(Line::from(Span::styled(
        "  SYSTEM",
        Style::default()
            .fg(theme::BRASS)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  load  ", Style::default().fg(theme::DIM)),
        Span::styled(
            format!("{:.2}  {:.2}  {:.2}", sys.load_avg[0], sys.load_avg[1], sys.load_avg[2]),
            Style::default().fg(theme::FG),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  mem   ", Style::default().fg(theme::DIM)),
        Span::styled(
            format!("{}%", sys.memory_pct as u64),
            Style::default().fg(if sys.memory_pct > 85.0 {
                theme::RED
            } else if sys.memory_pct > 70.0 {
                theme::YELLOW
            } else {
                theme::GREEN
            }),
        ),
        Span::styled(
            format!(" ({} / {})", format_mem(sys.memory_used_mb), format_mem(sys.memory_total_mb)),
            Style::default().fg(theme::DIM),
        ),
    ]));

    lines.push(Line::from(""));

    // CLAUDE section
    lines.push(Line::from(Span::styled(
        "  CLAUDE (today)",
        Style::default()
            .fg(theme::KEYS)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  msgs  ", Style::default().fg(theme::DIM)),
        Span::styled(format_num(claude.today_messages), Style::default().fg(theme::FG)),
        Span::styled("  sessions  ", Style::default().fg(theme::DIM)),
        Span::styled(format!("{}", claude.today_sessions), Style::default().fg(theme::FG)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  tools ", Style::default().fg(theme::DIM)),
        Span::styled(format_num(claude.today_tool_calls), Style::default().fg(theme::FG)),
    ]));

    // MODELS section
    if !claude.models.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  MODELS",
            Style::default()
                .fg(theme::CYAN)
                .add_modifier(Modifier::BOLD),
        )));
        for model in &claude.models {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", model.model), Style::default().fg(theme::FG)),
                Span::styled(
                    format!("{} in", format_tokens(model.input_tokens)),
                    Style::default().fg(theme::GREEN),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{} out", format_tokens(model.output_tokens)),
                    Style::default().fg(theme::YELLOW),
                ),
            ]));
        }
    }

    // ACTIVITY sparkline
    if !claude.hour_counts.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ACTIVITY (hourly)",
            Style::default()
                .fg(theme::SAX)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                sparkline(&claude.hour_counts),
                Style::default().fg(theme::BRASS),
            ),
        ]));
    }

    // AGENTS section
    if !agents.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  AGENTS",
            Style::default()
                .fg(theme::BRASS)
                .add_modifier(Modifier::BOLD),
        )));
        for agent in agents {
            let (icon, icon_color) = status_icon_and_color(&agent.status, &agent.process_status);
            let type_color = agent_type_color(&agent.agent_type);

            // Line 1: icon + name + status label
            let status_label = if agent.process_status == "dead" {
                "dead".to_string()
            } else {
                agent.status.clone()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(icon_color)),
                Span::styled(&agent.name, Style::default().fg(type_color)),
                Span::styled(
                    format!("  {}", status_label),
                    Style::default().fg(icon_color),
                ),
            ]));

            // Line 2: branch (indented under name)
            if !agent.branch.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("      ", Style::default()),
                    Span::styled(&agent.branch, Style::default().fg(theme::DIM)),
                ]));
            }

            // Line 3: task (if any)
            if !agent.task.is_empty() {
                let task_display = if agent.task.chars().count() > 30 {
                    let truncated: String = agent.task.chars().take(28).collect();
                    format!("{}..", truncated)
                } else {
                    agent.task.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled("      ", Style::default()),
                    Span::styled(task_display, Style::default().fg(theme::FG)),
                ]));
            }
        }
    }

    let content = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    frame.render_widget(content, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1000), "1.0K");
        assert_eq!(format_tokens(1500), "1.5K");
        assert_eq!(format_tokens(10000), "10K");
        assert_eq!(format_tokens(187000), "187K");
        assert_eq!(format_tokens(1000000), "1.0M");
        assert_eq!(format_tokens(1500000), "1.5M");
        assert_eq!(format_tokens(10000000), "10M");
    }

    #[test]
    fn test_format_num() {
        assert_eq!(format_num(0), "0");
        assert_eq!(format_num(123), "123");
        assert_eq!(format_num(1234), "1,234");
        assert_eq!(format_num(1234567), "1,234,567");
    }

    #[test]
    fn test_sparkline() {
        let counts = vec![0, 0, 0, 50, 100, 50, 0, 0];
        let s = sparkline(&counts);
        assert_eq!(s.chars().count(), 8);
        // First three should be space (0 value)
        assert_eq!(s.chars().next(), Some(' '));
        // Max value should be full block
        assert!(s.contains('\u{2588}'));
    }

    #[test]
    fn test_sparkline_empty() {
        let s = sparkline(&[]);
        assert!(s.is_empty());
    }

    #[test]
    fn test_sparkline_all_zeros() {
        let s = sparkline(&[0, 0, 0]);
        assert_eq!(s, "   ");
    }

    #[test]
    fn test_format_mem() {
        assert_eq!(format_mem(512), "512M");
        assert_eq!(format_mem(1024), "1.0G");
        assert_eq!(format_mem(16384), "16.0G");
    }
}
