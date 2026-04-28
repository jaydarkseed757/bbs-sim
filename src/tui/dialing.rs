use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

// Classic green-phosphor palette
const TERM_GREEN: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const TERM_DIM: Color   = Color::new(0.0, 0.45, 0.0, 1.0);

pub fn render_dialing(app: &App) {
    let d = match app.dial.as_ref() {
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

    // ── Header ────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);

    let hdr = format!(
        "  Dialing: {}  [ {} ]  @ {} baud",
        d.bbs.name, d.bbs.number, d.bbs.baud
    );
    draw_text_ex(
        &hdr,
        8.0,
        CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() },
    );

    // ── Body — terminal buffer ────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let rows_visible = ((body_h - 8.0) / CHAR_H) as usize;
    let origin_x = 12.0;
    let origin_y = body_y + 6.0;

    for (row, line) in d.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() {
            continue;
        }

        // Color: if the line looks like a clean modem command use bright green,
        // otherwise dim green for noise.
        let is_command = text.trim_start().starts_with("AT")
            || text.trim_start().starts_with("OK")
            || text.trim_start().starts_with("CONNECT")
            || text.trim_start().starts_with("NO ")
            || text.trim_start().starts_with("RING");

        let color = if is_command { TERM_GREEN } else { TERM_DIM };

        draw_text_ex(
            &text,
            origin_x,
            origin_y + (row + 1) as f32 * CHAR_H,
            TextParams { font_size: FONT_SIZE, color, ..Default::default() },
        );
    }

    // ── Footer — phase status ──────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);

    let phase_label = match d.modem.phase {
        crate::sim::modem::DialPhase::PickingUpLine => "Picking up line...",
        crate::sim::modem::DialPhase::Dialing       => "Dialing...",
        crate::sim::modem::DialPhase::Connecting    => "Connecting...",
        crate::sim::modem::DialPhase::Handshaking   => "Handshaking...",
        crate::sim::modem::DialPhase::Connected     => "Connected!",
        crate::sim::modem::DialPhase::NoAnswer      => "No answer.",
    };

    draw_text_ex(
        &format!("  {}", phase_label),
        8.0,
        footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() },
    );

    if d.awaiting_keypress {
        draw_text_ex(
            "Press [Enter] to log in",
            sw * 0.38,
            footer_y + CHAR_H * 1.3,
            TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() },
        );
    }

    draw_text_ex(
        "[Esc] Abort",
        sw - 130.0,
        footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() },
    );
}
