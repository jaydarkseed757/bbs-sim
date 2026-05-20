use macroquad::prelude::*;

/// Scanline + vignette overlay — call once per frame after all other drawing.
pub fn draw_crt_overlay() {
    let sw = screen_width();
    let sh = screen_height();

    // Scanlines: 1px dark stripe every 3 pixels
    let line_color = Color::new(0.0, 0.0, 0.0, 0.18);
    let mut y = 0.5_f32;
    while y < sh {
        draw_line(0.0, y, sw, y, 1.0, line_color);
        y += 3.0;
    }

    // Vignette: graduated dark bands on each edge
    let steps = 12usize;
    for i in 0..steps {
        let alpha = 0.28 * (1.0 - i as f32 / steps as f32).powi(2);
        let c = Color::new(0.0, 0.0, 0.0, alpha);
        let t = i as f32 * 3.5;
        // top
        draw_rectangle(0.0, t, sw, 3.5, c);
        // bottom
        draw_rectangle(0.0, sh - t - 3.5, sw, 3.5, c);
        // left
        draw_rectangle(t, 0.0, 3.5, sh, c);
        // right
        draw_rectangle(sw - t - 3.5, 0.0, 3.5, sh, c);
    }
}
