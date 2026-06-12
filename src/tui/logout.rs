use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt};

pub fn render_logout(app: &App) {
    let state = match app.logout.as_ref() {
        Some(s) => s,
        None => return,
    };

    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    txt(&format!("  {}  --  Logging Off", state.bbs_name), s(8.0), ch() * 1.3, t.title);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    let rows_visible = ((body_h - s(8.0)) / ch()) as usize;
    let origin_x = s(12.0);
    let origin_y = body_y + s(6.0);

    for (row, line) in state.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() { continue; }
        txt(&text, origin_x, origin_y + (row + 1) as f32 * ch(), app.theme.hi);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    let (msg, color) = if state.done {
        ("  Press any key to hang up...", t.title)
    } else {
        ("  Signing off...", t.muted)
    };
    txt(msg, s(8.0), footer_y + ch() * 1.3, color);
}
