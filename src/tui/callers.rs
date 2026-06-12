use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt, tw};

pub fn render_last_callers(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = format!(
        "  {}  --  LAST CALLERS  ({} recent)",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.last_callers.len(),
    );
    txt(&title, s(8.0), ch() * 1.3, t.title);

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    txt(
        "   #  HANDLE           LOCATION              TIME ON  DATE",
        s(12.0), body_y + ch() * 1.0,
        t.label,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), t.border);

    let row_h    = ch() * 1.5;
    let rows_vis = ((body_h - ch() * 1.5) / row_h) as usize;

    for (i, caller) in app.last_callers.iter().enumerate() {
        if i < app.callers_scroll { continue; }
        let slot = i - app.callers_scroll;
        if slot >= rows_vis { break; }

        let ry = body_y + ch() * 1.5 + slot as f32 * row_h;

        // Alternate row tint for readability.
        if slot.is_multiple_of(2) {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), row_h, Color::new(1.0, 1.0, 1.0, 0.03));
        }

        let num_color = match i {
            0 => t.title,
            1 | 2 => t.label,
            _ => t.muted,
        };

        txt(&format!("  {:>2}", i + 1), s(12.0), ry, num_color);
        txt(&format!("  {:<16}", clip(&caller.handle, 16)), s(50.0), ry, t.hi);
        txt(&format!("{:<22}", clip(&caller.location, 22)), s(222.0), ry, t.lo);
        txt(&caller.time_on, s(430.0), ry, t.muted);
        txt(&caller.date, s(495.0), ry, t.muted);
    }

    if app.last_callers.is_empty() {
        txt("  No caller records.", s(12.0), body_y + ch() * 3.0, t.muted);
    }

    // Scroll indicator
    if app.last_callers.len() > rows_vis {
        let max_scroll = app.last_callers.len().saturating_sub(rows_vis);
        let pct = (app.callers_scroll as f32 / max_scroll as f32 * 100.0) as u32;
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), t.muted);
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    let mut hx = s(8.0);
    for (key, desc) in &[("[↑↓/jk]", " Scroll  "), ("[Esc]", " Main Menu")] {
        txt(key, hx, footer_y + ch() * 1.2, t.title);
        hx += tw(key);
        txt(desc, hx, footer_y + ch() * 1.2, t.border);
        hx += tw(desc);
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}
