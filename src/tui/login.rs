use macroquad::prelude::*;

use crate::app::{App, LoginStep};

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const APPROX_CHAR_W: f32 = FONT_SIZE as f32 * 0.55; // good enough for cursor positioning

pub fn render_login(app: &App) {
    let d = match app.login.as_ref() {
        Some(d) => d,
        None => return,
    };

    let sw = screen_width();
    let sh = screen_height();

    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    draw_text_ex(
        &format!("  {}  [ Connected @ {} baud ]", d.bbs.name, d.bbs.baud),
        8.0,
        CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() },
    );

    // ── Body — terminal buffer ────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let rows_visible = ((body_h - 8.0) / CHAR_H) as usize;
    let origin_x = 12.0;
    let origin_y = body_y + 6.0;

    let total_lines = d.buffer.lines.len();
    let scroll_start = total_lines.saturating_sub(rows_visible);

    for (row, line) in d.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() {
            continue;
        }

        // Use the style of the first non-whitespace character on the line.
        let color = line
            .iter()
            .find(|sc| !sc.ch.is_whitespace())
            .map(|sc| sc.style.fg)
            .unwrap_or(Color::new(0.85, 0.85, 0.85, 1.0));

        draw_text_ex(
            &text,
            origin_x,
            origin_y + (row + 1) as f32 * CHAR_H,
            TextParams { font_size: FONT_SIZE, color, ..Default::default() },
        );
    }

    // ── Blinking block cursor ─────────────────────────────────────────────────
    let in_input = matches!(d.step, LoginStep::Handle | LoginStep::Password);
    if in_input && d.cursor_blink {
        let cursor_visible_row = d.buffer.cursor_row.saturating_sub(scroll_start);
        if cursor_visible_row < rows_visible {
            let cx = origin_x + d.buffer.cursor_col as f32 * APPROX_CHAR_W;
            let cy = origin_y + cursor_visible_row as f32 * CHAR_H + 3.0;
            draw_rectangle(cx, cy, APPROX_CHAR_W, CHAR_H - 2.0, Color::new(0.0, 0.75, 0.0, 0.85));
        }
    }

    // ── Footer — step status ──────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);

    let step_label = match d.step {
        LoginStep::Banner | LoginStep::HandlePrompt => "  Receiving BBS info...",
        LoginStep::Handle   => "  Enter your handle",
        LoginStep::PassPrompt => "  ...",
        LoginStep::Password => "  Enter your password",
        LoginStep::WelcomeText => "  Logging in...",
        LoginStep::Done     => "  Welcome!",
    };

    draw_text_ex(
        step_label,
        8.0,
        footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() },
    );
    draw_text_ex(
        "[Esc] Disconnect",
        sw - 165.0,
        footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() },
    );
}
