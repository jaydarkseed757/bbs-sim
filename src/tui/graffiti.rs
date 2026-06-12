use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, cw, s, txt, tw};

pub fn render_graffiti_wall(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let input_h  = if app.graffiti_input.is_some() { ch() * 2.8 } else { 0.0 };
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h - input_h;
    let footer_y = sh - footer_h;
    let input_y  = footer_y - input_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = format!(
        "  {}  --  GRAFFITI WALL  ({} line{})",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.oneliners.len(),
        if app.oneliners.len() == 1 { "" } else { "s" },
    );
    txt(&title, s(8.0), ch() * 1.3, t.title);

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    let rows_vis    = ((body_h - s(8.0)) / ch()) as usize;
    let total       = app.oneliners.len();
    let max_scroll  = total.saturating_sub(rows_vis);
    let scroll      = app.graffiti_scroll.min(max_scroll);

    if total == 0 {
        txt("  No entries yet. Press [A] to be the first!", s(12.0), body_y + ch() * 2.0, t.muted);
    }

    for (i, entry) in app.oneliners[scroll..total.min(scroll + rows_vis)].iter().enumerate() {
        let ry = body_y + s(4.0) + (i as f32 + 1.0) * ch();
        let handle_text = format!("  {}: ", entry.handle);
        let hw = tw(&handle_text);

        txt(&handle_text, s(8.0), ry, t.label);
        txt(
            &entry.message, s(8.0) + hw, ry,
            if i % 2 == 0 { t.hi } else { t.body },
        );
        txt(&entry.date, sw - s(80.0), ry, t.muted);
    }

    if total > rows_vis {
        let pct = if max_scroll == 0 { 100 } else { (scroll as f32 / max_scroll as f32 * 100.0) as u32 };
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), t.muted);
    }

    // ── Input area ────────────────────────────────────────────────────────────
    if let Some(ref text) = app.graffiti_input {
        draw_rectangle_lines(s(1.0), input_y, sw - s(2.0), input_h, s(1.5), t.hi);
        let handle = app.session.user_handle.as_deref().unwrap_or("You");
        let label  = format!("  {}: ", handle);
        let lw     = tw(&label);

        txt(&label, s(8.0), input_y + ch() * 1.4, t.label);
        txt(text, s(8.0) + lw, input_y + ch() * 1.4, t.hi);

        let text_w = tw(text);
        draw_rectangle(
            s(8.0) + lw + text_w, input_y + ch() * 1.4 - ch() + s(4.0),
            cw(), ch() - s(2.0),
            t.cursor,
        );

        let remaining = format!("{}/60 chars", text.len());
        txt(&remaining, sw - s(100.0), input_y + ch() * 1.4, t.muted);
        txt("  [Enter] Post  [Esc] Cancel", s(8.0), input_y + ch() * 2.4, t.lo);
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    if app.graffiti_input.is_none() {
        let mut hx = s(8.0);
        for (key, desc) in &[
            ("[↑↓/jk]", " Scroll  "),
            ("[A]", " Add Entry  "),
            ("[Esc]", " Main Menu"),
        ] {
            txt(key, hx, footer_y + ch() * 1.2, t.title);
            hx += tw(key);
            txt(desc, hx, footer_y + ch() * 1.2, t.border);
            hx += tw(desc);
        }
    }
}
