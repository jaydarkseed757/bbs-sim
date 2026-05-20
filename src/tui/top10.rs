use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt, tw};

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const CYAN_BBS:  Color = Color::new(0.0, 0.87, 0.87, 1.0);
const DIM_GRAY:  Color = Color::new(0.35, 0.35, 0.35, 1.0);
const GOLD:      Color = Color::new(1.0, 0.84, 0.0, 1.0);
const SILVER:    Color = Color::new(0.75, 0.75, 0.75, 1.0);
const BRONZE:    Color = Color::new(0.8, 0.5, 0.2, 1.0);

const TABS: &[&str] = &["Top Callers", "Top Downloads", "Top Posters"];

pub fn render_top10(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let tab_h    = ch() * 1.8;
    let footer_h = ch() * 2.2;
    let body_y   = header_h + tab_h;
    let body_h   = sh - body_y - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(
        &format!("  {}  --  TOP 10 LISTS", app.session.bbs_name.as_deref().unwrap_or("BBS")),
        s(8.0), ch() * 1.3,
        YELLOW,
    );

    // ── Tab bar ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), header_h, sw - s(2.0), tab_h, s(1.5), DARKGRAY);
    let mut tx = s(16.0);
    for (i, label) in TABS.iter().enumerate() {
        let active = i == app.top_list_tab;
        let padded = format!("  {}  ", label);
        let w = tw(&padded);
        if active {
            draw_rectangle(tx - s(2.0), header_h + s(2.0), w + s(4.0), tab_h - s(4.0),
                Color::new(0.0, 0.25, 0.0, 1.0));
            draw_rectangle_lines(tx - s(2.0), header_h + s(2.0), w + s(4.0), tab_h - s(4.0), s(1.0), GREEN_BBS);
        }
        txt(&padded, tx, header_h + ch() * 1.1, if active { GREEN_BBS } else { DIM_GRAY });
        tx += w + s(8.0);
    }
    txt("[Tab] Switch", sw - s(120.0), header_h + ch() * 1.1, DIM_GRAY);

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let lists = &app.top_lists;
    match app.top_list_tab {
        0 => render_callers(lists.callers.iter().take(10).collect(), body_y, sw),
        1 => render_files(lists.files.iter().take(10).collect(), body_y, sw),
        2 => render_posters(lists.posters.iter().take(10).collect(), body_y, sw),
        _ => {}
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    let mut hx = s(8.0);
    for (key, desc) in &[("[Tab]", " Switch list  "), ("[Esc]", " Main Menu")] {
        txt(key, hx, footer_y + ch() * 1.2, YELLOW);
        hx += tw(key);
        txt(desc, hx, footer_y + ch() * 1.2, DARKGRAY);
        hx += tw(desc);
    }
}

fn rank_color(rank: usize) -> Color {
    match rank {
        0 => GOLD,
        1 => SILVER,
        2 => BRONZE,
        _ => DIM_GREEN,
    }
}

fn render_callers(callers: Vec<&crate::bbs::data::TopCaller>, body_y: f32, sw: f32) {
    if callers.is_empty() {
        txt("  No data.", s(12.0), body_y + ch() * 2.0, DIM_GRAY);
        return;
    }
    txt("   #   HANDLE               CALLS    LOCATION", s(12.0), body_y + ch() * 1.0, CYAN_BBS);
    draw_line(s(8.0), body_y + ch() * 1.25, sw - s(8.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, c) in callers.iter().enumerate() {
        let ry  = body_y + ch() * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i);
        let rank_str = rank_label(i);
        txt(&rank_str,             s(12.0), ry, col);
        txt(&format!("{:<20}", c.handle), s(70.0), ry, GREEN_BBS);
        txt(&format!("{:>6}", c.calls),   s(290.0), ry, col);
        txt(&c.location,           s(360.0), ry, DIM_GREEN);
    }
}

fn render_files(files: Vec<&crate::bbs::data::TopFile>, body_y: f32, sw: f32) {
    if files.is_empty() {
        txt("  No data.", s(12.0), body_y + ch() * 2.0, DIM_GRAY);
        return;
    }
    txt("   #   FILENAME              DLs     DESCRIPTION", s(12.0), body_y + ch() * 1.0, CYAN_BBS);
    draw_line(s(8.0), body_y + ch() * 1.25, sw - s(8.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, f) in files.iter().enumerate() {
        let ry  = body_y + ch() * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i);
        let rank_str = rank_label(i);
        txt(&rank_str,                    s(12.0), ry, col);
        txt(&format!("{:<20}", f.name),   s(70.0), ry, GREEN_BBS);
        txt(&format!("{:>6}", f.downloads), s(290.0), ry, col);
        txt(&f.description,               s(360.0), ry, DIM_GREEN);
    }
}

fn render_posters(posters: Vec<&crate::bbs::data::TopPoster>, body_y: f32, sw: f32) {
    if posters.is_empty() {
        txt("  No data.", s(12.0), body_y + ch() * 2.0, DIM_GRAY);
        return;
    }
    txt("   #   HANDLE               POSTS    BOARD", s(12.0), body_y + ch() * 1.0, CYAN_BBS);
    draw_line(s(8.0), body_y + ch() * 1.25, sw - s(8.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, p) in posters.iter().enumerate() {
        let ry  = body_y + ch() * (i as f32 * 1.6 + 2.4);
        let col = rank_color(i);
        let rank_str = rank_label(i);
        txt(&rank_str,                    s(12.0), ry, col);
        txt(&format!("{:<20}", p.handle), s(70.0), ry, GREEN_BBS);
        txt(&format!("{:>6}", p.posts),   s(290.0), ry, col);
        txt(&p.board,                     s(360.0), ry, DIM_GREEN);
    }
}

fn rank_label(i: usize) -> String {
    match i {
        0 => " #1 ★".to_string(),
        1 => " #2  ".to_string(),
        2 => " #3  ".to_string(),
        n => format!(" #{:<2} ", n + 1),
    }
}
