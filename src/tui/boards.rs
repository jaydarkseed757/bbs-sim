use macroquad::prelude::*;

use crate::app::App;
use crate::bbs::data::Thread;
use crate::tui::font::{ch, s, txt, tw, BODY_PAD};
use crate::tui::theme::BbsTheme;

/// Thread reader chrome, in ch() units — shared with the scroll clamp in app.rs.
pub const READ_HEADER_CH: f32 = 2.2;
pub const READ_FOOTER_CH: f32 = 2.2;

// ── Message Boards list ───────────────────────────────────────────────────────

pub fn render_message_boards(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = format!(
        "  {}  --  MESSAGE BOARDS",
        app.session.bbs_name.as_deref().unwrap_or("BBS")
    );
    txt(&title, s(8.0), ch() * 1.3, t.title);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);
    txt(
        "   #  BOARD NAME                         THREADS",
        s(12.0), body_y + ch() * 1.0,
        t.label,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), t.border);

    for (i, board) in app.boards.iter().enumerate() {
        let ry = body_y + ch() * (i as f32 * 1.5 + 2.2);
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_board_row;
        if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0), t.sel_bg);
        }
        let fg     = if selected { t.hi } else { t.lo };
        let prefix = if selected { "►" } else { " " };
        let line   = format!("  {} {:2}  {:<40}  {}", prefix, i + 1, board.name, board.threads.len());
        txt(&line, s(12.0), ry, fg);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Open Board  "),
        ("[Esc]", " Main Menu"),
    ], t.title, t.border);
}

// ── Thread list ───────────────────────────────────────────────────────────────

pub fn render_thread_list(app: &App, board_id: &str) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let board = match app.boards.iter().find(|b| b.id == board_id) {
        Some(b) => b,
        None => return,
    };

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = format!("  Board: {}  ({} threads)", board.name, board.threads.len());
    txt(&title, s(8.0), ch() * 1.3, t.title);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);
    txt(
        "   #  SUBJECT                                  FROM          DATE     RPL",
        s(12.0), body_y + ch() * 1.0,
        t.label,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), t.border);

    for (i, thread) in board.threads.iter().enumerate() {
        let ry = body_y + ch() * (i as f32 * 1.5 + 2.2);
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_thread_row;
        if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0), t.sel_bg);
        }
        let fg     = if selected { t.hi } else { t.lo };
        let prefix = if selected { "►" } else { " " };
        let first   = &thread.posts[0];
        let replies = thread.posts.len() - 1;
        let date    = if first.timestamp.len() >= 8 { &first.timestamp[..8] } else { &first.timestamp };
        let line = format!(
            "  {} {:2}  {:<44}  {:<12}  {}  {}",
            prefix, i + 1,
            clip(&thread.subject, 44),
            clip(&first.author, 12),
            date,
            replies,
        );
        txt(&line, s(12.0), ry, fg);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Read Thread  "),
        ("[N]", " New Thread  "),
        ("[Esc]", " Boards"),
    ], t.title, t.border);
}

// ── Read thread ───────────────────────────────────────────────────────────────

pub fn render_read_thread(app: &App, board_id: &str, thread_id: u32) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * READ_HEADER_CH;
    let footer_h = ch() * READ_FOOTER_CH;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let thread = match find_thread(&app.boards, board_id, thread_id) {
        Some(t) => t,
        None => return,
    };

    let lines      = build_thread_lines(thread, t);
    let rows_vis   = ((body_h - s(BODY_PAD)) / ch()) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.read_scroll.min(max_scroll);

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    txt(&format!("  {}", clip(&thread.subject, 70)), s(8.0), ch() * 1.3, t.title);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    let visible = &lines[scroll..lines.len().min(scroll + rows_vis)];
    for (i, (text, color)) in visible.iter().enumerate() {
        let y = body_y + s(8.0) + (i as f32 + 1.0) * ch();
        txt(text, s(12.0), y, *color);
    }

    if lines.len() > rows_vis {
        let pct = if max_scroll == 0 { 100 }
                  else { (scroll as f32 / max_scroll as f32 * 100.0) as u32 };
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), t.muted);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓]", " Scroll  "),
        ("[PgUp/PgDn]", " Page  "),
        ("[R]", " Reply  "),
        ("[Esc]", " Back"),
    ], t.title, t.border);
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

fn build_thread_lines(thread: &Thread, t: &BbsTheme) -> Vec<(String, Color)> {
    let sep = "-".repeat(78);
    let mut out: Vec<(String, Color)> = vec![];

    for (i, msg) in thread.posts.iter().enumerate() {
        if i > 0 {
            out.push((String::new(), t.body));
            out.push((sep.clone(), t.muted));
        }
        out.push((
            format!("From : {}   Date: {}   Msg #{}", msg.author, msg.timestamp, msg.id),
            t.label,
        ));
        out.push((format!("Subj : {}", msg.subject), t.label));
        out.push((String::new(), t.body));
        for body_line in msg.body.replace('\r', "").split('\n') {
            out.push((body_line.to_string(), t.body));
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

fn draw_hint_bar(y: f32, hints: &[(&str, &str)], key_color: Color, desc_color: Color) {
    let mut hx = s(8.0);
    for (key, desc) in hints {
        txt(key, hx, y, key_color);
        hx += tw(key);
        txt(desc, hx, y, desc_color);
        hx += tw(desc);
    }
}
