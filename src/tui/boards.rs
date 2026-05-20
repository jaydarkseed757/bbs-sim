use macroquad::prelude::*;

use crate::app::App;
use crate::bbs::data::Thread;
use crate::tui::font::{ch, s, txt, tw};

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const CYAN_BBS:  Color = Color::new(0.0, 0.87, 0.87, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);
const DIM_GRAY:  Color = Color::new(0.35, 0.35, 0.35, 1.0);

// ── Message Boards list ───────────────────────────────────────────────────────

pub fn render_message_boards(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    let title = format!(
        "  {}  --  MESSAGE BOARDS",
        app.session.bbs_name.as_deref().unwrap_or("BBS")
    );
    txt(&title, s(8.0), ch() * 1.3, YELLOW);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);
    txt(
        "   #  BOARD NAME                         THREADS",
        s(12.0), body_y + ch() * 1.0,
        CYAN_BBS,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, board) in app.boards.iter().enumerate() {
        let ry = body_y + ch() * (i as f32 * 1.5 + 2.2);
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_board_row;
        if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0),
                Color::new(0.0, 0.25, 0.0, 1.0));
        }
        let fg     = if selected { GREEN_BBS } else { DIM_GREEN };
        let prefix = if selected { "►" } else { " " };
        let line   = format!("  {} {:2}  {:<40}  {}", prefix, i + 1, board.name, board.threads.len());
        txt(&line, s(12.0), ry, fg);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Open Board  "),
        ("[Esc]", " Main Menu"),
    ]);
}

// ── Thread list ───────────────────────────────────────────────────────────────

pub fn render_thread_list(app: &App, board_id: &str) {
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

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    let title = format!("  Board: {}  ({} threads)", board.name, board.threads.len());
    txt(&title, s(8.0), ch() * 1.3, YELLOW);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);
    txt(
        "   #  SUBJECT                                  FROM          DATE     RPL",
        s(12.0), body_y + ch() * 1.0,
        CYAN_BBS,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, thread) in board.threads.iter().enumerate() {
        let ry = body_y + ch() * (i as f32 * 1.5 + 2.2);
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_thread_row;
        if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0),
                Color::new(0.0, 0.25, 0.0, 1.0));
        }
        let fg     = if selected { GREEN_BBS } else { DIM_GREEN };
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

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Read Thread  "),
        ("[N]", " New Thread  "),
        ("[Esc]", " Boards"),
    ]);
}

// ── Read thread ───────────────────────────────────────────────────────────────

pub fn render_read_thread(app: &App, board_id: &str, thread_id: u32) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let thread = match find_thread(&app.boards, board_id, thread_id) {
        Some(t) => t,
        None => return,
    };

    let lines      = build_thread_lines(thread);
    let rows_vis   = ((body_h - s(16.0)) / ch()) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.read_scroll.min(max_scroll);

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(&format!("  {}", clip(&thread.subject, 70)), s(8.0), ch() * 1.3, YELLOW);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let visible = &lines[scroll..lines.len().min(scroll + rows_vis)];
    for (i, (text, color)) in visible.iter().enumerate() {
        let y = body_y + s(8.0) + (i as f32 + 1.0) * ch();
        txt(text, s(12.0), y, *color);
    }

    if lines.len() > rows_vis {
        let pct = (scroll as f32 / max_scroll as f32 * 100.0) as u32;
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), DIM_GRAY);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓]", " Scroll  "),
        ("[PgUp/PgDn]", " Page  "),
        ("[R]", " Reply  "),
        ("[Esc]", " Back"),
    ]);
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

fn build_thread_lines(thread: &Thread) -> Vec<(String, Color)> {
    let sep = "-".repeat(78);
    let mut out: Vec<(String, Color)> = vec![];

    for (i, msg) in thread.posts.iter().enumerate() {
        if i > 0 {
            out.push((String::new(), WHITE_BBS));
            out.push((sep.clone(), DIM_GRAY));
        }
        out.push((
            format!("From : {}   Date: {}   Msg #{}", msg.author, msg.timestamp, msg.id),
            CYAN_BBS,
        ));
        out.push((format!("Subj : {}", msg.subject), CYAN_BBS));
        out.push((String::new(), WHITE_BBS));
        for body_line in msg.body.replace('\r', "").split('\n') {
            out.push((body_line.to_string(), WHITE_BBS));
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

fn draw_hint_bar(y: f32, hints: &[(&str, &str)]) {
    let mut hx = s(8.0);
    for (key, desc) in hints {
        txt(key, hx, y, YELLOW);
        hx += tw(key);
        txt(desc, hx, y, DARKGRAY);
        hx += tw(desc);
    }
}
