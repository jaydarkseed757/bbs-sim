use macroquad::prelude::*;

use crate::app::{App, ComposeMailField};
use crate::tui::font::{ch, cw, s, txt, tw};

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);
const CYAN_BBS:  Color = Color::new(0.0, 0.87, 0.87, 1.0);
const DIM_GRAY:  Color = Color::new(0.35, 0.35, 0.35, 1.0);

// ── Inbox ─────────────────────────────────────────────────────────────────────

pub fn render_inbox(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    let unread: usize = app.mail.iter().filter(|m| !m.read).count();
    let title = format!(
        "  {}  --  MAIL  ({} message{}, {} new)",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.mail.len(),
        if app.mail.len() == 1 { "" } else { "s" },
        unread,
    );
    txt(&title, s(8.0), ch() * 1.3, YELLOW);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);
    txt(
        "   #  ST  FROM                 SUBJECT                           DATE",
        s(12.0), body_y + ch() * 1.0,
        CYAN_BBS,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    for (i, msg) in app.mail.iter().enumerate() {
        let ry = body_y + ch() * (i as f32 * 1.5 + 2.2);
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_mail_row;
        if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0),
                Color::new(0.0, 0.25, 0.0, 1.0));
        }

        let fg     = if selected { GREEN_BBS } else { DIM_GREEN };
        let prefix = if selected { "►" } else { " " };
        let status = if msg.read { "  " } else { "* " };
        let date   = if msg.timestamp.len() >= 8 { &msg.timestamp[..8] } else { &msg.timestamp };
        let line   = format!(
            "  {} {:2}  {}  {:<20}  {:<34}  {}",
            prefix, i + 1, status,
            clip(&msg.from, 20),
            clip(&msg.subject, 34),
            date,
        );
        txt(&line, s(12.0), ry, fg);
    }

    if app.mail.is_empty() {
        txt("  No messages.", s(12.0), body_y + ch() * 3.0, DIM_GRAY);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓/jk]", " Navigate  "),
        ("[Enter]", " Read  "),
        ("[C]", " Compose  "),
        ("[Esc]", " Main Menu"),
    ]);
}

// ── Read mail ─────────────────────────────────────────────────────────────────

pub fn render_read_mail(app: &App, id: u32) {
    let msg = match app.mail.iter().find(|m| m.id == id) {
        Some(m) => m,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 4.0;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    let cy0 = ch() * 0.9;
    txt(&format!("  From : {}", msg.from),    s(8.0), cy0 + ch() * 0.8, CYAN_BBS);
    txt(&format!("  To   : {}", msg.to),      s(8.0), cy0 + ch() * 1.7, CYAN_BBS);
    txt(&format!("  Subj : {}", msg.subject), s(8.0), cy0 + ch() * 2.6, YELLOW);
    txt(&format!("  Date : {}", msg.timestamp), sw * 0.5, cy0 + ch() * 0.8, DIM_GREEN);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let lines: Vec<&str> = msg.body.split('\n').collect();
    let rows_vis   = ((body_h - s(16.0)) / ch()) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.mail_read_scroll.min(max_scroll);

    for (i, line) in lines[scroll..lines.len().min(scroll + rows_vis)].iter().enumerate() {
        let y = body_y + s(8.0) + (i as f32 + 1.0) * ch();
        txt(line, s(12.0), y, WHITE_BBS);
    }

    if lines.len() > rows_vis {
        let pct = (scroll as f32 / max_scroll as f32 * 100.0) as u32;
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), DIM_GRAY);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓]", " Scroll  "),
        ("[R]", " Reply  "),
        ("[Esc]", " Inbox"),
    ]);
}

// ── Compose mail ──────────────────────────────────────────────────────────────

pub fn render_compose_mail(app: &App) {
    let compose = match app.compose_mail.as_ref() {
        Some(c) => c,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    let title = if compose.reply_to_id.is_some() {
        format!("  Compose: Reply to {}", compose.to)
    } else {
        "  Compose: New Mail".to_string()
    };
    txt(&title, s(8.0), ch() * 1.3, YELLOW);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let mut cur_y = body_y + ch() * 1.3;
    cur_y = draw_field("  To      : ", &compose.to, compose.field == ComposeMailField::To, cur_y, sw);
    cur_y += ch() * 0.5;
    cur_y = draw_field("  Subject : ", &compose.subject, compose.field == ComposeMailField::Subject, cur_y, sw);
    cur_y += ch() * 0.9;

    txt("  Message:", s(8.0), cur_y, CYAN_BBS);
    cur_y += ch() * 0.9;

    let editor_x = s(12.0);
    let editor_y = cur_y;
    let editor_w = sw - s(24.0);
    let editor_h = footer_y - editor_y - s(8.0);
    let active   = compose.field == ComposeMailField::Body;

    draw_rectangle_lines(editor_x - s(2.0), editor_y, editor_w + s(4.0), editor_h, s(1.0),
        if active { GREEN_BBS } else { DARKGRAY });

    let rows_vis  = ((editor_h - s(8.0)) / ch()) as usize;
    let body_lines: Vec<&str> = compose.body.split('\n').collect();
    let total  = body_lines.len();
    let scroll = total.saturating_sub(rows_vis);

    for (i, line) in body_lines[scroll..].iter().enumerate() {
        let y = editor_y + s(6.0) + (i as f32 + 1.0) * ch();
        if y > footer_y - s(8.0) { break; }
        txt(line, editor_x + s(4.0), y, WHITE_BBS);
    }

    if active {
        let last = body_lines.last().copied().unwrap_or("");
        let vis_row = total - 1 - scroll;
        let cy = editor_y + s(6.0) + (vis_row as f32 + 1.0) * ch();
        if cy < footer_y - s(8.0) {
            let cx = editor_x + s(4.0) + tw(last);
            draw_rectangle(cx, cy - ch() + s(4.0), cw(), ch() - s(2.0),
                Color::new(0.0, 0.75, 0.0, 0.8));
        }
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[Tab]", " Next Field  "),
        ("[F1]", " Send  "),
        ("[Esc]", " Abort"),
    ]);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_field(label: &str, value: &str, active: bool, y: f32, sw: f32) -> f32 {
    let lw = tw(label);
    txt(label, s(8.0), y, CYAN_BBS);
    let fx = s(8.0) + lw;
    let fw = sw - fx - s(16.0);
    draw_rectangle_lines(fx - s(2.0), y - ch() + s(2.0), fw, ch() + s(4.0), s(1.0),
        if active { GREEN_BBS } else { DARKGRAY });
    txt(value, fx + s(2.0), y, GREEN_BBS);
    if active {
        let cx = fx + s(2.0) + tw(value);
        draw_rectangle(cx, y - ch() + s(4.0), cw(), ch() - s(2.0),
            Color::new(0.0, 0.75, 0.0, 0.8));
    }
    y + ch() * 1.3
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
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
