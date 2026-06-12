use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt};


pub fn render_dialing(app: &App) {
    let d = match app.dial.as_ref() {
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

    // ── Header ────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);

    let hdr = format!(
        "  Dialing: {}  [ {} ]  @ {} baud",
        d.bbs.name, d.bbs.number, d.bbs.baud
    );
    txt(&hdr, s(8.0), ch() * 1.3, YELLOW);

    // ── Body — terminal buffer ────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let rows_visible = ((body_h - s(8.0)) / ch()) as usize;
    let origin_x = s(12.0);
    let origin_y = body_y + s(6.0);

    for (row, line) in d.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() { continue; }

        let is_command = text.trim_start().starts_with("AT")
            || text.trim_start().starts_with("OK")
            || text.trim_start().starts_with("CONNECT")
            || text.trim_start().starts_with("NO ")
            || text.trim_start().starts_with("RING");

        let color = if is_command { app.theme.hi } else { app.theme.lo };
        txt(&text, origin_x, origin_y + (row + 1) as f32 * ch(), color);
    }

    // ── Footer — phase status ──────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);

    let dots = ".".repeat((get_time() * 2.5) as usize % 4);
    let phase_label = match d.modem.phase {
        crate::sim::modem::DialPhase::PickingUpLine => format!("Picking up line{}", dots),
        crate::sim::modem::DialPhase::Dialing       => format!("Dialing{}", dots),
        crate::sim::modem::DialPhase::Connecting    => format!("Connecting{}", dots),
        crate::sim::modem::DialPhase::Handshaking   => format!("Handshaking{}", dots),
        crate::sim::modem::DialPhase::Connected     => "Connected!".to_string(),
        crate::sim::modem::DialPhase::NoAnswer      => "No answer.".to_string(),
    };

    txt(&format!("  {}", phase_label), s(8.0), footer_y + ch() * 1.3, DARKGRAY);

    if d.awaiting_keypress {
        txt("Press [Enter] to log in", sw * 0.38, footer_y + ch() * 1.3, YELLOW);
    }

    txt("[Esc] Abort", sw - s(130.0), footer_y + ch() * 1.3, DARKGRAY);
}
