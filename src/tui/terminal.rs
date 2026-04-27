#![allow(dead_code)]

use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

#[derive(Clone)]
pub struct StyledChar {
    pub ch: char,
    pub style: Style,
}

impl StyledChar {
    pub fn plain(ch: char) -> Self {
        Self {
            ch,
            style: Style::default(),
        }
    }

    pub fn styled(ch: char, style: Style) -> Self {
        Self { ch, style }
    }
}

/// Scrolling character buffer for BBS terminal output.
pub struct TerminalBuffer {
    pub lines: Vec<Vec<StyledChar>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub width: usize,
    pub height: usize,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            lines: vec![vec![]],
            cursor_row: 0,
            cursor_col: 0,
            width,
            height,
        }
    }

    pub fn push_char(&mut self, ch: char, style: Style) {
        match ch {
            '\r' => {
                self.cursor_col = 0;
            }
            '\n' => {
                self.lines.push(vec![]);
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            _ => {
                while self.lines.len() <= self.cursor_row {
                    self.lines.push(vec![]);
                }
                let line = &mut self.lines[self.cursor_row];
                while line.len() < self.cursor_col {
                    line.push(StyledChar::plain(' '));
                }
                if self.cursor_col < line.len() {
                    line[self.cursor_col] = StyledChar::styled(ch, style);
                } else {
                    line.push(StyledChar::styled(ch, style));
                }
                self.cursor_col += 1;
            }
        }
    }

    pub fn push_str(&mut self, s: &str, style: Style) {
        for ch in s.chars() {
            self.push_char(ch, style);
        }
    }

    pub fn as_text(&self) -> Text<'static> {
        let lines: Vec<Line<'static>> = self
            .lines
            .iter()
            .map(|row| {
                let spans: Vec<Span<'static>> = row
                    .iter()
                    .map(|sc| Span::styled(sc.ch.to_string(), sc.style))
                    .collect();
                Line::from(spans)
            })
            .collect();
        Text::from(lines)
    }

    /// Returns the last `height` lines for rendering.
    pub fn visible_lines(&self, height: usize) -> &[Vec<StyledChar>] {
        if self.lines.len() <= height {
            &self.lines
        } else {
            &self.lines[self.lines.len() - height..]
        }
    }
}
