#![allow(dead_code)]

use macroquad::prelude::*;

use crate::app::App;

pub fn render_main_menu(_app: &App) {
    draw_rectangle_lines(1.0, 1.0, screen_width() - 2.0, screen_height() - 2.0, 1.5, GREEN);
    draw_text_ex(
        " MAIN MENU ",
        8.0,
        22.0,
        TextParams { font_size: 18, color: GREEN, ..Default::default() },
    );
    draw_text_ex(
        "  Main menu -- coming soon.",
        16.0,
        48.0,
        TextParams { font_size: 18, color: WHITE, ..Default::default() },
    );
}
