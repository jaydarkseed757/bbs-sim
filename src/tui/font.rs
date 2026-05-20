use std::cell::RefCell;
use macroquad::prelude::*;

// Logical (unscaled) base sizes.  Everything is multiplied by dpi_scale() at draw time.
pub const BASE_FS: f32 = 18.0;

thread_local! {
    static BBS_FONT: RefCell<Option<Font>> = RefCell::new(None);
}

pub fn set_bbs_font(font: Font) {
    BBS_FONT.with(|f| *f.borrow_mut() = Some(font));
}

/// Font size in logical pixels (macroquad's coordinate space).
#[inline] pub fn fs() -> u16 { BASE_FS as u16 }

/// Line height in logical pixels.
#[inline] pub fn ch() -> f32 { BASE_FS * 1.35 }

/// Approximate character width for Monaco (~0.60 of height).
#[inline] pub fn cw() -> f32 { BASE_FS * 0.60 }

/// Pass-through; callers use s() so switching DPI models only needs this one change.
#[inline] pub fn s(px: f32) -> f32 { px }

/// Draw text with the BBS font at the standard size.
pub fn txt(text: &str, x: f32, y: f32, color: Color) {
    BBS_FONT.with(|f| {
        let guard = f.borrow();
        draw_text_ex(text, x, y, TextParams {
            font: guard.as_ref(),
            font_size: fs(),
            color,
            ..Default::default()
        });
    });
}

/// Draw text at a custom (already-scaled) size.
pub fn txt_sz(text: &str, x: f32, y: f32, size: u16, color: Color) {
    BBS_FONT.with(|f| {
        let guard = f.borrow();
        draw_text_ex(text, x, y, TextParams {
            font: guard.as_ref(),
            font_size: size,
            color,
            ..Default::default()
        });
    });
}

/// Measure text width with the BBS font at the standard size.
pub fn tw(text: &str) -> f32 {
    BBS_FONT.with(|f| {
        let guard = f.borrow();
        measure_text(text, guard.as_ref(), fs(), 1.0).width
    })
}

/// Measure text width at a custom size.
pub fn tw_sz(text: &str, size: u16) -> f32 {
    BBS_FONT.with(|f| {
        let guard = f.borrow();
        measure_text(text, guard.as_ref(), size, 1.0).width
    })
}
