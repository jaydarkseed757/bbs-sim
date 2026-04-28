use macroquad::prelude::*;

use crate::app::App;
use crate::bbs::data::Thread;
use crate::tui::theme::Theme;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

// ── Message Boards list ───────────────────────────────────────────────────────

pub fn render_message_boards(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!(
        "  {}  --  MESSAGE BOARDS",
        app.session.bbs_name.as_deref().unwrap_or("BBS")
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);
    draw_text_ex(
        "   #  BOARD NAME                         THREADS",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() },
    );
    draw_line(4.0, body_y + CHAR_H * 1.25, sw - 4.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    for (i, board) in app.boards.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.5 + 2.2);
        if ry + CHAR_H > footer_y { break; }

        let selected = i == app.selected_board_row;
        if selected {
            draw_rectangle(2.0, ry - CHAR_H + 4.0, sw - 4.0, CHAR_H + 2.0, t.sel_bg);
        }
        let fg     = if selected { t.primary } else { t.secondary };
        let prefix = if selected { "►" } else { " " };
        let line   = format!("  {} {:2}  {:<40}  {}",
            prefix, i + 1, board.name, board.threads.len());
        draw_text_ex(&line, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: fg, ..Default::default() });
    }

    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Open Board  "),
        ("[Esc]", " Main Menu"),
    ], t);
}

// ── Thread list ───────────────────────────────────────────────────────────────

pub fn render_thread_list(app: &App, board_id: &str) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let board = match app.boards.iter().find(|b| b.id == board_id) {
        Some(b) => b,
        None => return,
    };

    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!("  Board: {}  ({} threads)", board.name, board.threads.len());
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);
    draw_text_ex(
        "   #  SUBJECT                                  FROM          DATE     RPL",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() },
    );
    draw_line(4.0, body_y + CHAR_H * 1.25, sw - 4.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    for (i, thread) in board.threads.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.5 + 2.2);
        if ry + CHAR_H > footer_y { break; }

        let selected = i == app.selected_thread_row;
        if selected {
            draw_rectangle(2.0, ry - CHAR_H + 4.0, sw - 4.0, CHAR_H + 2.0, t.sel_bg);
        }
        let fg     = if selected { t.primary } else { t.secondary };
        let prefix = if selected { "►" } else { " " };
        let first   = &thread.posts[0];
        let replies = thread.posts.len() - 1;
        let date    = if first.timestamp.len() >= 8 { &first.timestamp[..8] } else { &first.timestamp };
        let line    = format!(
            "  {} {:2}  {:<44}  {:<12}  {}  {}",
            prefix, i + 1,
            clip(&thread.subject, 44),
            clip(&first.author, 12),
            date,
            replies,
        );
        draw_text_ex(&line, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: fg, ..Default::default() });
    }

    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Read Thread  "),
        ("[N]", " New Thread  "),
        ("[Esc]", " Boards"),
    ], t);
}

// ── Read thread ───────────────────────────────────────────────────────────────

pub fn render_read_thread(app: &App, board_id: &str, thread_id: u32) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let thread = match find_thread(&app.boards, board_id, thread_id) {
        Some(t) => t,
        None => return,
    };

    let lines       = build_thread_lines(thread, t);
    let rows_vis    = ((body_h - 16.0) / CHAR_H) as usize;
    let max_scroll  = lines.len().saturating_sub(rows_vis);
    let scroll      = app.read_scroll.min(max_scroll);

    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    draw_text_ex(
        &format!("  {}", clip(&thread.subject, 70)),
        8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() },
    );

    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    let visible = &lines[scroll..lines.len().min(scroll + rows_vis)];
    for (i, (text, color)) in visible.iter().enumerate() {
        let y = body_y + 8.0 + (i as f32 + 1.0) * CHAR_H;
        draw_text_ex(text, 12.0, y,
            TextParams { font_size: FONT_SIZE, color: *color, ..Default::default() });
    }

    if lines.len() > rows_vis {
        let pct = (scroll as f32 / max_scroll as f32 * 100.0) as u32;
        draw_text_ex(
            &format!("{:3}%", pct),
            sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() },
        );
    }

    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
        ("[↑↓]", " Scroll  "),
        ("[PgUp/PgDn]", " Page  "),
        ("[R]", " Reply  "),
        ("[Esc]", " Back"),
    ], t);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_thread<'a>(
    boards: &'a [crate::bbs::data::Board],
    board_id: &str,
    thread_id: u32,
) -> Option<&'a Thread> {
    boards.iter()
        .find(|b| b.id == board_id)?
        .threads.iter()
        .find(|t| t.id == thread_id)
}

fn build_thread_lines(thread: &Thread, t: &Theme) -> Vec<(String, Color)> {
    let sep = "-".repeat(78);
    let mut out: Vec<(String, Color)> = vec![];

    for (i, msg) in thread.posts.iter().enumerate() {
        if i > 0 {
            out.push((String::new(), t.primary));
            out.push((sep.clone(), t.dim));
        }
        out.push((
            format!("From : {}   Date: {}   Msg #{}", msg.author, msg.timestamp, msg.id),
            t.highlight,
        ));
        out.push((format!("Subj : {}", msg.subject), t.highlight));
        out.push((String::new(), t.primary));
        for body_line in msg.body.replace('\r', "").split('\n') {
            out.push((body_line.to_string(), t.primary));
        }
    }
    out
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}~", truncated)
    }
}

fn draw_hint_bar(y: f32, hints: &[(&str, &str)], t: &Theme) {
    let mut hx = 8.0;
    for (key, desc) in hints {
        draw_text_ex(key, hx, y,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, y,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}
