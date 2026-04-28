use macroquad::prelude::*;

use crate::app::App;
use crate::tui::theme::Theme;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

const GOLD:   Color = Color::new(1.0,  0.84, 0.0,  1.0);
const SILVER: Color = Color::new(0.75, 0.75, 0.75, 1.0);
const BRONZE: Color = Color::new(0.8,  0.5,  0.2,  1.0);

const TABS: &[&str] = &["Top Callers", "Top Downloads", "Top Posters"];

pub fn render_top10(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let tab_h    = CHAR_H * 1.8;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h + tab_h;
    let body_h   = sh - body_y - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!(
        "  {}  --  TOP 10 LISTS",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Tab bar ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, header_h, sw - 2.0, tab_h, 1.5, t.border);
    let mut tx = 16.0;
    for (i, label) in TABS.iter().enumerate() {
        let active = i == app.top_list_tab;
        let padded = format!("  {}  ", label);
        let w = measure_text(&padded, None, FONT_SIZE, 1.0).width;
        if active {
            draw_rectangle(tx - 2.0, header_h + 2.0, w + 4.0, tab_h - 4.0, t.sel_bg);
            draw_rectangle_lines(tx - 2.0, header_h + 2.0, w + 4.0, tab_h - 4.0, 1.0, t.primary);
        }
        draw_text_ex(&padded, tx, header_h + CHAR_H * 1.1,
            TextParams { font_size: FONT_SIZE,
                color: if active { t.primary } else { t.dim },
                ..Default::default() });
        tx += w + 8.0;
    }
    draw_text_ex("[Tab] Switch", sw - 120.0, header_h + CHAR_H * 1.1,
        TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    let lists = &app.top_lists;
    match app.top_list_tab {
        0 => render_callers(lists.callers.iter().take(10).collect(), body_y, sw, t),
        1 => render_files(lists.files.iter().take(10).collect(), body_y, sw, t),
        2 => render_posters(lists.posters.iter().take(10).collect(), body_y, sw, t),
        _ => {}
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    let mut hx = 8.0;
    for (key, desc) in &[("[Tab]", " Switch list  "), ("[Esc]", " Main Menu")] {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

fn rank_color(rank: usize, t: &Theme) -> Color {
    match rank {
        0 => GOLD,
        1 => SILVER,
        2 => BRONZE,
        _ => t.secondary,
    }
}

fn render_callers(callers: Vec<&crate::bbs::data::TopCaller>, body_y: f32, sw: f32, t: &Theme) {
    if callers.is_empty() {
        draw_text_ex("  No data.", 12.0, body_y + CHAR_H * 2.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
        return;
    }
    draw_text_ex(
        "   #   HANDLE               CALLS    LOCATION",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    draw_line(8.0, body_y + CHAR_H * 1.25, sw - 8.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    for (i, c) in callers.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i, t);
        let rank_str = match i {
            0 => " #1 ★".to_string(),
            1 => " #2  ".to_string(),
            2 => " #3  ".to_string(),
            n => format!(" #{:<2} ", n + 1),
        };
        draw_text_ex(&rank_str, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&format!("{:<20}", c.handle), 70.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        draw_text_ex(&format!("{:>6}", c.calls), 290.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&c.location, 360.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
    }
}

fn render_files(files: Vec<&crate::bbs::data::TopFile>, body_y: f32, sw: f32, t: &Theme) {
    if files.is_empty() {
        draw_text_ex("  No data.", 12.0, body_y + CHAR_H * 2.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
        return;
    }
    draw_text_ex(
        "   #   FILENAME              DLs     DESCRIPTION",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    draw_line(8.0, body_y + CHAR_H * 1.25, sw - 8.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    for (i, f) in files.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i, t);
        let rank_str = match i {
            0 => " #1 ★".to_string(),
            1 => " #2  ".to_string(),
            2 => " #3  ".to_string(),
            n => format!(" #{:<2} ", n + 1),
        };
        draw_text_ex(&rank_str, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&format!("{:<20}", f.name), 70.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        draw_text_ex(&format!("{:>6}", f.downloads), 290.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&f.description, 360.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
    }
}

fn render_posters(posters: Vec<&crate::bbs::data::TopPoster>, body_y: f32, sw: f32, t: &Theme) {
    if posters.is_empty() {
        draw_text_ex("  No data.", 12.0, body_y + CHAR_H * 2.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
        return;
    }
    draw_text_ex(
        "   #   HANDLE               POSTS    BOARD",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    draw_line(8.0, body_y + CHAR_H * 1.25, sw - 8.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    for (i, p) in posters.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i, t);
        let rank_str = match i {
            0 => " #1 ★".to_string(),
            1 => " #2  ".to_string(),
            2 => " #3  ".to_string(),
            n => format!(" #{:<2} ", n + 1),
        };
        draw_text_ex(&rank_str, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&format!("{:<20}", p.handle), 70.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        draw_text_ex(&format!("{:>6}", p.posts), 290.0, ry,
            TextParams { font_size: FONT_SIZE, color: col, ..Default::default() });
        draw_text_ex(&p.board, 360.0, ry,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
    }
}
