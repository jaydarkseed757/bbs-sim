use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, cw, s, txt, tw};

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);

pub fn render_main_menu(app: &App) {
    let sw = screen_width();
    let sh = screen_height();

    let handle   = app.session.user_handle.as_deref().unwrap_or("Guest");
    let bbs_name = app.session.bbs_name.as_deref().unwrap_or("BBS");

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), sh - s(2.0), s(1.5), DARKGRAY);

    // ── Top banner ─────────────────────────────────────────────────────────────
    let banner_h = ch() * 3.5;
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), banner_h, s(1.5), DARKGRAY);

    txt(bbs_name, s(16.0), ch() * 1.4, YELLOW);
    txt(&format!("Logged in as: {}", handle), s(16.0), ch() * 2.6, DIM_GREEN);

    // ── Menu items ─────────────────────────────────────────────────────────────
    let menu_x  = sw * 0.20;
    let start_y = banner_h + ch() * 2.5;

    let items: &[(&str, &str, &str)] = &[
        ("[B]", "oards",      "Read and post on message boards"),
        ("[M]", "ail",        "Read and send private mail"),
        ("[F]", "iles",       "Browse & download files"),
        ("[O]", "ne-liners",  "Graffiti wall -- leave your mark"),
        ("[T]", "op 10",      "Top callers, downloads, and posters"),
        ("[C]", "hat",        "Chat with the sysop"),
        ("[G]", "oodbye",     "Logoff the system"),
    ];

    for (i, (key, rest, hint)) in items.iter().enumerate() {
        let ry = start_y + i as f32 * ch() * 2.2;

        txt(key, menu_x, ry, YELLOW);
        let key_w = tw(key);
        txt(rest, menu_x + key_w, ry, GREEN_BBS);
        let label_w = key_w + tw(rest);
        txt(&format!("  --  {}", hint), menu_x + label_w, ry, DIM_GREEN);
    }

    // ── Prompt ─────────────────────────────────────────────────────────────────
    let prompt_y = start_y + items.len() as f32 * ch() * 2.2 + ch() * 1.5;
    txt("Choice: ", menu_x, prompt_y, WHITE_BBS);

    if app.blink {
        let cw_val = tw("Choice: ");
        draw_rectangle(
            menu_x + cw_val, prompt_y - ch() + s(4.0),
            cw(), ch() - s(2.0),
            Color::new(0.0, 0.75, 0.0, 0.8),
        );
    }
}
