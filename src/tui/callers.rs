use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

pub fn render_last_callers(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!(
        "  {}  --  LAST CALLERS  ({} recent)",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.last_callers.len(),
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    draw_text_ex(
        "   #  HANDLE           LOCATION              TIME ON  DATE",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() },
    );
    draw_line(4.0, body_y + CHAR_H * 1.25, sw - 4.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    let row_h    = CHAR_H * 1.5;
    let rows_vis = ((body_h - CHAR_H * 1.5) / row_h) as usize;

    for (i, caller) in app.last_callers.iter().enumerate() {
        if i < app.callers_scroll { continue; }
        let slot = i - app.callers_scroll;
        if slot >= rows_vis { break; }

        let ry = body_y + CHAR_H * 1.5 + slot as f32 * row_h;

        // Alternate row tint for readability.
        if slot.is_multiple_of(2) {
            draw_rectangle(2.0, ry - CHAR_H + 4.0, sw - 4.0, row_h, Color::new(1.0, 1.0, 1.0, 0.03));
        }

        let num_color = match i {
            0 => t.title,
            1 | 2 => t.highlight,
            _ => t.dim,
        };

        draw_text_ex(&format!("  {:>2}", i + 1), 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: num_color, ..Default::default() });
        draw_text_ex(&format!("  {:<16}", clip(&caller.handle, 16)), 50.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        draw_text_ex(&format!("{:<22}", clip(&caller.location, 22)), 222.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
        draw_text_ex(&caller.time_on, 430.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
        draw_text_ex(&caller.date, 495.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    if app.last_callers.is_empty() {
        draw_text_ex("  No caller records.", 12.0, body_y + CHAR_H * 3.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    // Scroll indicator
    if app.last_callers.len() > rows_vis {
        let max_scroll = app.last_callers.len().saturating_sub(rows_vis);
        let pct = (app.callers_scroll as f32 / max_scroll as f32 * 100.0) as u32;
        draw_text_ex(&format!("{:3}%", pct), sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    let mut hx = 8.0;
    for (key, desc) in &[("[↑↓/jk]", " Scroll  "), ("[Esc]", " Main Menu")] {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}
