use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, Tab};
use crate::tabs;
use crate::theme;

pub fn render(frame: &mut Frame, app: &App) {
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
    let tabs: Vec<Span> = Tab::ALL
        .iter()
        .enumerate()
        .flat_map(|(i, tab)| {
            let label = tab.label();
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

            if i < 3 {
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

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.active_tab {
        Tab::Players => tabs::players::render(frame, area, app),
        Tab::Charts => tabs::charts::render(frame, area, app),
        Tab::Beads => tabs::beads::render(frame, area, app),
        Tab::Setlist => tabs::setlist::render(frame, area, app),
    }
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let totals = &app.state.totals;
    let agents = &app.state.agents;

    let running = agents.iter().filter(|a| a.status == "running").count();
    let waiting = agents.iter().filter(|a| a.status == "waiting").count();
    let idle = agents
        .iter()
        .filter(|a| a.status != "running" && a.status != "waiting" && a.status != "error")
        .count();

    let tok_str = if totals.tokens > 1000 {
        format!("{}K", totals.tokens / 1000)
    } else {
        format!("{}", totals.tokens)
    };

    let footer_text = if agents.is_empty() {
        " \u{2669}0  [a]dd  [q]uit".to_string()
    } else {
        format!(
            " \u{2669}{}  \u{1d13e}{}  \u{25cb}{}  tok:{}  ${:.2}",
            running, waiting, idle, tok_str, totals.cost_usd
        )
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        footer_text,
        Style::default().fg(theme::DIM),
    )))
    .style(Style::default().bg(theme::BG_ALT));
    frame.render_widget(footer, area);
}
