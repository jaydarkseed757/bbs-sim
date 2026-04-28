use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const CHAR_W: f32 = FONT_SIZE as f32 * 0.55;

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);
const CYAN_BBS:  Color = Color::new(0.0, 0.87, 0.87, 1.0);
const DIM_GRAY:  Color = Color::new(0.35, 0.35, 0.35, 1.0);

pub fn render_graffiti_wall(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h  = CHAR_H * 2.2;
    let footer_h  = CHAR_H * 2.2;
    let input_h   = if app.graffiti_input.is_some() { CHAR_H * 2.8 } else { 0.0 };
    let body_y    = header_h;
    let body_h    = sh - header_h - footer_h - input_h;
    let footer_y  = sh - footer_h;
    let input_y   = footer_y - input_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    let title = format!(
        "  {}  --  GRAFFITI WALL  ({} line{})",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.oneliners.len(),
        if app.oneliners.len() == 1 { "" } else { "s" },
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let rows_vis = ((body_h - 8.0) / CHAR_H) as usize;
    let total    = app.oneliners.len();
    let max_scroll = total.saturating_sub(rows_vis);
    let scroll   = app.graffiti_scroll.min(max_scroll);

    if total == 0 {
        draw_text_ex("  No entries yet. Press [A] to be the first!",
            12.0, body_y + CHAR_H * 2.0,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() });
    }

    for (i, entry) in app.oneliners[scroll..total.min(scroll + rows_vis)].iter().enumerate() {
        let ry = body_y + 4.0 + (i as f32 + 1.0) * CHAR_H;
        let handle_text = format!("  {}: ", entry.handle);
        let hw = measure_text(&handle_text, None, FONT_SIZE, 1.0).width;

        draw_text_ex(&handle_text, 8.0, ry,
            TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });
        draw_text_ex(&entry.message, 8.0 + hw, ry,
            TextParams { font_size: FONT_SIZE, color: if i % 2 == 0 { GREEN_BBS } else { WHITE_BBS }, ..Default::default() });
        draw_text_ex(&entry.date, sw - 80.0, ry,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() });
    }

    // scroll indicator
    if total > rows_vis {
        let pct = if max_scroll == 0 { 100 } else { (scroll as f32 / max_scroll as f32 * 100.0) as u32 };
        draw_text_ex(&format!("{:3}%", pct), sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() });
    }

    // ── Input area ────────────────────────────────────────────────────────────
    if let Some(ref text) = app.graffiti_input {
        draw_rectangle_lines(1.0, input_y, sw - 2.0, input_h, 1.5, GREEN_BBS);
        let handle = app.session.user_handle.as_deref().unwrap_or("You");
        let label  = format!("  {}: ", handle);
        let lw     = measure_text(&label, None, FONT_SIZE, 1.0).width;

        draw_text_ex(&label, 8.0, input_y + CHAR_H * 1.4,
            TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });
        draw_text_ex(text, 8.0 + lw, input_y + CHAR_H * 1.4,
            TextParams { font_size: FONT_SIZE, color: GREEN_BBS, ..Default::default() });

        // cursor
        let tw = measure_text(text, None, FONT_SIZE, 1.0).width;
        draw_rectangle(8.0 + lw + tw, input_y + CHAR_H * 1.4 - CHAR_H + 4.0,
            CHAR_W, CHAR_H - 2.0, Color::new(0.0, 0.75, 0.0, 0.8));

        let remaining = format!("{}/60 chars", text.len());
        draw_text_ex(&remaining, sw - 100.0, input_y + CHAR_H * 1.4,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() });

        draw_text_ex("  [Enter] Post  [Esc] Cancel", 8.0, input_y + CHAR_H * 2.4,
            TextParams { font_size: FONT_SIZE, color: DIM_GREEN, ..Default::default() });
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    let mut hx = 8.0;
    let hints: &[(&str, &str)] = if app.graffiti_input.is_some() {
        &[]
    } else {
        &[
            ("[↑↓/jk]", " Scroll  "),
            ("[A]", " Add Entry  "),
            ("[Esc]", " Main Menu"),
        ]
    };
    for (key, desc) in hints {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}
