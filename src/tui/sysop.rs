use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const TERM_GREEN: Color = Color::new(0.0, 0.85, 0.0, 1.0);

pub fn render_sysop_chat(app: &App) {
    let state = match app.sysop_chat.as_ref() {
        Some(s) => s,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    draw_text_ex(
        &format!("  {}  --  CHAT WITH SYSOP", state.bbs_name),
        8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() },
    );

    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let rows_visible = ((body_h - 8.0) / CHAR_H) as usize;
    let origin_x = 12.0;
    let origin_y = body_y + 6.0;

    for (row, line) in state.buffer.visible_lines(rows_visible).iter().enumerate() {
        let text: String = line.iter().map(|sc| sc.ch).collect();
        if text.trim().is_empty() { continue; }
        draw_text_ex(
            &text,
            origin_x,
            origin_y + (row + 1) as f32 * CHAR_H,
            TextParams { font_size: FONT_SIZE, color: TERM_GREEN, ..Default::default() },
        );
    }

    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    let (msg, color) = if state.done {
        ("  Press any key to return to main menu...", YELLOW)
    } else {
        ("  Contacting sysop...", DARKGRAY)
    };
    draw_text_ex(msg, 8.0, footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color, ..Default::default() });
}
