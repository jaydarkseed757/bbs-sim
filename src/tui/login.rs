use macroquad::prelude::*;

use crate::app::{App, LoginStep};
use crate::tui::font::{ch, cw, s, txt};

pub fn render_login(app: &App) {
    let d = match app.login.as_ref() {
        Some(d) => d,
        None => return,
    };

    let sw = screen_width();
    let sh = screen_height();

    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(
        &format!("  {}  [ Connected @ {} baud ]", d.bbs.name, d.bbs.baud),
        s(8.0), ch() * 1.3,
        YELLOW,
    );

    // ── Body — terminal buffer ────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let rows_visible = ((body_h - s(8.0)) / ch()) as usize;
    let origin_x = s(12.0);
    let origin_y = body_y + s(6.0);

    let total_lines = d.buffer.lines.len();
    let scroll_start = total_lines.saturating_sub(rows_visible);

    for (row, line) in d.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() { continue; }

        let color = line
            .iter()
            .find(|sc| !sc.ch.is_whitespace())
            .map(|sc| sc.style.fg)
            .unwrap_or(Color::new(0.85, 0.85, 0.85, 1.0));

        txt(&text, origin_x, origin_y + (row + 1) as f32 * ch(), color);
    }

    // ── Blinking block cursor ─────────────────────────────────────────────────
    let in_input = matches!(d.step, LoginStep::Handle | LoginStep::Password);
    if in_input && d.cursor_blink {
        let cursor_visible_row = d.buffer.cursor_row.saturating_sub(scroll_start);
        if cursor_visible_row < rows_visible {
            let cx = origin_x + d.buffer.cursor_col as f32 * cw();
            let cy = origin_y + cursor_visible_row as f32 * ch() + s(3.0);
            draw_rectangle(cx, cy, cw(), ch() - s(2.0), Color::new(0.0, 0.75, 0.0, 0.85));
        }
    }

    // ── Footer — step status ──────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);

    let step_label = match d.step {
        LoginStep::Banner | LoginStep::HandlePrompt => "  Receiving BBS info...",
        LoginStep::Handle   => "  Enter your handle",
        LoginStep::PassPrompt => "  ...",
        LoginStep::Password => "  Enter your password",
        LoginStep::WelcomeText => "  Logging in...",
        LoginStep::Done     => "  Welcome!",
    };

    txt(step_label, s(8.0), footer_y + ch() * 1.3, DARKGRAY);
    txt("[Esc] Disconnect", sw - s(165.0), footer_y + ch() * 1.3, DARKGRAY);
}
