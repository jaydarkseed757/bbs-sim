use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);

pub fn render_main_menu(app: &App) {
    let sw = screen_width();
    let sh = screen_height();

    let handle   = app.session.user_handle.as_deref().unwrap_or("Guest");
    let bbs_name = app.session.bbs_name.as_deref().unwrap_or("BBS");

    // Outer border
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, sh - 2.0, 1.5, DARKGRAY);

    // ── Top banner ─────────────────────────────────────────────────────────────
    let banner_h = CHAR_H * 3.5;
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, banner_h, 1.5, DARKGRAY);

    draw_text_ex(
        bbs_name,
        16.0, CHAR_H * 1.4,
        TextParams { font_size: FONT_SIZE + 4, color: YELLOW, ..Default::default() },
    );
    draw_text_ex(
        &format!("Logged in as: {}", handle),
        16.0, CHAR_H * 2.6,
        TextParams { font_size: FONT_SIZE, color: DIM_GREEN, ..Default::default() },
    );

    // ── Menu items ─────────────────────────────────────────────────────────────
    let menu_x  = sw * 0.20;
    let start_y = banner_h + CHAR_H * 2.5;

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
        let ry = start_y + i as f32 * CHAR_H * 2.2;

        // Key letter
        draw_text_ex(key, menu_x, ry,
            TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });
        // Rest of label
        let key_w = measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(rest, menu_x + key_w, ry,
            TextParams { font_size: FONT_SIZE, color: GREEN_BBS, ..Default::default() });
        // Hint
        let label_w = key_w + measure_text(rest, None, FONT_SIZE, 1.0).width;
        draw_text_ex(
            &format!("  --  {}", hint),
            menu_x + label_w, ry,
            TextParams { font_size: FONT_SIZE, color: DIM_GREEN, ..Default::default() },
        );
    }

    // ── Prompt ─────────────────────────────────────────────────────────────────
    let prompt_y = start_y + items.len() as f32 * CHAR_H * 2.2 + CHAR_H * 1.5;
    draw_text_ex(
        "Choice: ",
        menu_x, prompt_y,
        TextParams { font_size: FONT_SIZE, color: WHITE_BBS, ..Default::default() },
    );
    // Blinking block cursor
    if app.blink {
        let cw = measure_text("Choice: ", None, FONT_SIZE, 1.0).width;
        draw_rectangle(
            menu_x + cw, prompt_y - CHAR_H + 4.0,
            FONT_SIZE as f32 * 0.55, CHAR_H - 2.0,
            Color::new(0.0, 0.75, 0.0, 0.8),
        );
    }
}
