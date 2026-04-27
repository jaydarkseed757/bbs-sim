use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

const HEADER_ART: &str = concat!(
    "  ██████╗ ██████╗ ███████╗       ███████╗██╗███╗   ███╗\n",
    "  ██╔══██╗██╔══██╗██╔════╝       ██╔════╝██║████╗ ████║\n",
    "  ██████╔╝██████╔╝███████╗ █████╗███████╗██║██╔████╔██║\n",
    "  ██╔══██╗██╔══██╗╚════██║ ╚════╝╚════██║██║██║╚██╔╝██║\n",
    "  ██████╔╝██████╔╝███████║       ███████║██║██║ ╚═╝ ██║\n",
    "  ╚═════╝ ╚═════╝ ╚══════╝       ╚══════╝╚═╝╚═╝     ╚═╝"
);

pub fn render_dialer(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // ASCII art header
            Constraint::Min(5),     // phonebook table
            Constraint::Length(3),  // help bar
        ])
        .split(area);

    // ── Header ──────────────────────────────────────────────────────────────
    let header = Paragraph::new(HEADER_ART)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("  BBS-SIM v0.1.0 — PHONEBOOK DIALER  ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .style(Style::default().fg(Color::Green)),
        );
    frame.render_widget(header, chunks[0]);

    // ── Phonebook table ──────────────────────────────────────────────────────
    let col_header = Row::new([
        Cell::from("  BBS NAME").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("PHONE NUMBER").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("BAUD").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Cell::from("LAST CALLED").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .phonebook
        .iter()
        .map(|entry| {
            let last = entry.last_called.as_deref().unwrap_or("Never");
            Row::new([
                Cell::from(format!("  {}", entry.name)),
                Cell::from(entry.number.clone()),
                Cell::from(format!("{}", entry.baud)),
                Cell::from(last.to_string()),
            ])
            .style(Style::default().fg(Color::Green))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(42),
            Constraint::Percentage(22),
            Constraint::Percentage(12),
            Constraint::Percentage(24),
        ],
    )
    .header(col_header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" PHONEBOOK ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .style(Style::default().fg(Color::DarkGray)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("►  ");

    frame.render_stateful_widget(table, chunks[1], &mut app.dialer_state);

    // ── Help bar ─────────────────────────────────────────────────────────────
    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            "  [↑↓/jk]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[D]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Dial  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[A]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Add  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Del]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Remove  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Q/Esc]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(help, chunks[2]);
}
