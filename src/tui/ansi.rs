use macroquad::prelude::Color;

use crate::tui::terminal::CellStyle;

pub struct AnsiParser {
    fg:   Color,
    bg:   Option<Color>,
    bold: bool,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self { fg: ansi_color(7), bg: None, bold: false }
    }

    /// Parse ANSI-escaped text into (char, CellStyle) pairs.
    /// Only SGR ('m') sequences are handled; all other CSI sequences are stripped.
    pub fn parse(&mut self, input: &str) -> Vec<(char, CellStyle)> {
        let mut out   = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                if let Some(&'[') = chars.peek() {
                    chars.next(); // consume '['
                    let mut params = String::new();
                    loop {
                        match chars.next() {
                            None => break,
                            Some(c) if c.is_ascii_digit() || c == ';' => {
                                params.push(c);
                            }
                            Some('m') => {
                                apply_sgr(self, &params);
                                break;
                            }
                            Some(_) => break, // non-SGR sequence — discard
                        }
                    }
                }
                // lone ESC — discard
            } else {
                out.push((ch, self.style()));
            }
        }
        out
    }

    fn style(&self) -> CellStyle {
        CellStyle { fg: self.fg, bg: self.bg, bold: self.bold }
    }
}

fn apply_sgr(p: &mut AnsiParser, params: &str) {
    let codes: Vec<&str> = if params.is_empty() { vec!["0"] } else { params.split(';').collect() };
    for code in codes {
        let n: u8 = code.parse().unwrap_or(0);
        match n {
            0        => { p.fg = ansi_color(7); p.bg = None; p.bold = false; }
            1        => p.bold = true,
            22       => p.bold = false,
            39       => p.fg = ansi_color(7),
            49       => p.bg = None,
            30..=37  => p.fg = ansi_color(n - 30),
            90..=97  => p.fg = ansi_color(n - 90 + 8),
            40..=47  => p.bg = Some(ansi_color(n - 40)),
            100..=107 => p.bg = Some(ansi_color(n - 100 + 8)),
            _        => {}
        }
    }
}

fn ansi_color(n: u8) -> Color {
    match n {
        0  => Color::new(0.0,  0.0,  0.0,  1.0),
        1  => Color::new(0.67, 0.0,  0.0,  1.0),
        2  => Color::new(0.0,  0.67, 0.0,  1.0),
        3  => Color::new(0.67, 0.67, 0.0,  1.0),
        4  => Color::new(0.0,  0.0,  0.67, 1.0),
        5  => Color::new(0.67, 0.0,  0.67, 1.0),
        6  => Color::new(0.0,  0.67, 0.67, 1.0),
        7  => Color::new(0.67, 0.67, 0.67, 1.0),
        8  => Color::new(0.33, 0.33, 0.33, 1.0),
        9  => Color::new(1.0,  0.33, 0.33, 1.0),
        10 => Color::new(0.33, 1.0,  0.33, 1.0),
        11 => Color::new(1.0,  1.0,  0.33, 1.0),
        12 => Color::new(0.33, 0.33, 1.0,  1.0),
        13 => Color::new(1.0,  0.33, 1.0,  1.0),
        14 => Color::new(0.33, 1.0,  1.0,  1.0),
        15 => Color::new(1.0,  1.0,  1.0,  1.0),
        _  => Color::new(0.67, 0.67, 0.67, 1.0),
    }
}
