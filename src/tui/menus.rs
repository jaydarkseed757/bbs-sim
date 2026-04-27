#![allow(dead_code, unused_variables)]

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render_main_menu(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" MAIN MENU ")
        .style(Style::default().fg(Color::Green));
    let text = Paragraph::new(Line::from(Span::raw(
        "  Main menu — coming soon.",
    )))
    .block(block);
    frame.render_widget(text, area);
}
