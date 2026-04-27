#![allow(dead_code)]

use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// CellStyle — replaces ratatui::style::Style
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
}

impl CellStyle {
    pub fn plain() -> Self {
        Self { fg: GREEN, bg: None, bold: false }
    }

    pub fn fg(color: Color) -> Self {
        Self { fg: color, bg: None, bold: false }
    }
}

// ---------------------------------------------------------------------------
// StyledChar
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct StyledChar {
    pub ch: char,
    pub style: CellStyle,
}

impl StyledChar {
    pub fn plain(ch: char) -> Self {
        Self { ch, style: CellStyle::plain() }
    }

    pub fn styled(ch: char, style: CellStyle) -> Self {
        Self { ch, style }
    }
}

// ---------------------------------------------------------------------------
// TerminalBuffer
// ---------------------------------------------------------------------------

/// Scrolling character grid for BBS terminal output.
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

    pub fn push_char(&mut self, ch: char, style: CellStyle) {
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

    pub fn push_str(&mut self, s: &str, style: CellStyle) {
        for ch in s.chars() {
            self.push_char(ch, style);
        }
    }

    /// Returns the last `height` lines for rendering.
    pub fn visible_lines(&self, height: usize) -> &[Vec<StyledChar>] {
        if self.lines.len() <= height {
            &self.lines
        } else {
            &self.lines[self.lines.len() - height..]
        }
    }

    /// Draw the buffer contents using macroquad.
    pub fn draw(
        &self,
        origin_x: f32,
        origin_y: f32,
        char_w: f32,
        char_h: f32,
        font: Option<&Font>,
        font_size: u16,
        rows_visible: usize,
    ) {
        for (row, line) in self.visible_lines(rows_visible).iter().enumerate() {
            for (col, sc) in line.iter().enumerate() {
                let x = origin_x + col as f32 * char_w;
                let y = origin_y + row as f32 * char_h + char_h; // baseline offset

                if let Some(bg) = sc.style.bg {
                    draw_rectangle(x, y - char_h, char_w, char_h, bg);
                }

                let s = sc.ch.to_string();
                draw_text_ex(
                    &s,
                    x,
                    y,
                    TextParams {
                        font,
                        font_size,
                        color: sc.style.fg,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
