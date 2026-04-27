use macroquad::prelude::*;

use crate::app::{App, ComposeMailField};

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const CHAR_W: f32 = FONT_SIZE as f32 * 0.55;

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);
const CYAN_BBS: Color  = Color::new(0.0, 0.87, 0.87, 1.0);
const DIM_GRAY: Color  = Color::new(0.35, 0.35, 0.35, 1.0);

// ── Inbox ─────────────────────────────────────────────────────────────────────

pub fn render_inbox(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    let unread: usize = app.mail.iter().filter(|m| !m.read).count();
    let title = format!(
        "  {}  --  MAIL  ({} message{}, {} new)",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.mail.len(),
        if app.mail.len() == 1 { "" } else { "s" },
        unread,
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });

    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);
    draw_text_ex(
        "   #  ST  FROM                 SUBJECT                           DATE",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() },
    );
    draw_line(4.0, body_y + CHAR_H * 1.25, sw - 4.0, body_y + CHAR_H * 1.25, 1.0, DARKGRAY);

    for (i, msg) in app.mail.iter().enumerate() {
        let ry = body_y + CHAR_H * (i as f32 * 1.5 + 2.2);
        if ry + CHAR_H > footer_y { break; }

        let selected = i == app.selected_mail_row;
        if selected {
            draw_rectangle(2.0, ry - CHAR_H + 4.0, sw - 4.0, CHAR_H + 2.0,
                Color::new(0.0, 0.25, 0.0, 1.0));
        }

        let fg     = if selected { GREEN_BBS } else { DIM_GREEN };
        let prefix = if selected { "►" } else { " " };
        let status = if msg.read { "  " } else { "* " };
        let date   = if msg.timestamp.len() >= 8 { &msg.timestamp[..8] } else { &msg.timestamp };
        let line   = format!(
            "  {} {:2}  {}  {:<20}  {:<34}  {}",
            prefix,
            i + 1,
            status,
            clip(&msg.from, 20),
            clip(&msg.subject, 34),
            date,
        );
        draw_text_ex(&line, 12.0, ry,
            TextParams { font_size: FONT_SIZE, color: fg, ..Default::default() });
    }

    if app.mail.is_empty() {
        draw_text_ex(
            "  No messages.",
            12.0, body_y + CHAR_H * 3.0,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() },
        );
    }

    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
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
    let header_h = CHAR_H * 4.0;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header — envelope info ─────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    let cy0 = CHAR_H * 0.9;
    draw_text_ex(&format!("  From : {}", msg.from), 8.0, cy0 + CHAR_H * 0.8,
        TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });
    draw_text_ex(&format!("  To   : {}", msg.to), 8.0, cy0 + CHAR_H * 1.7,
        TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });
    draw_text_ex(&format!("  Subj : {}", msg.subject), 8.0, cy0 + CHAR_H * 2.6,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });
    draw_text_ex(&format!("  Date : {}", msg.timestamp),
        sw * 0.5, cy0 + CHAR_H * 0.8,
        TextParams { font_size: FONT_SIZE, color: DIM_GREEN, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let lines: Vec<&str> = msg.body.split('\n').collect();
    let rows_vis   = ((body_h - 16.0) / CHAR_H) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.mail_read_scroll.min(max_scroll);

    for (i, line) in lines[scroll..lines.len().min(scroll + rows_vis)].iter().enumerate() {
        let y = body_y + 8.0 + (i as f32 + 1.0) * CHAR_H;
        draw_text_ex(line, 12.0, y,
            TextParams { font_size: FONT_SIZE, color: WHITE_BBS, ..Default::default() });
    }

    if lines.len() > rows_vis {
        let pct = (scroll as f32 / max_scroll as f32 * 100.0) as u32;
        draw_text_ex(&format!("{:3}%", pct), sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: DIM_GRAY, ..Default::default() });
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
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
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    let title = if compose.reply_to_id.is_some() {
        format!("  Compose: Reply to {}", compose.to)
    } else {
        "  Compose: New Mail".to_string()
    };
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });

    // ── Form ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);

    let mut cur_y = body_y + CHAR_H * 1.3;

    // To field
    cur_y = draw_field(
        "  To      : ", &compose.to,
        compose.field == ComposeMailField::To,
        cur_y, sw,
    );
    cur_y += CHAR_H * 0.5;

    // Subject field
    cur_y = draw_field(
        "  Subject : ", &compose.subject,
        compose.field == ComposeMailField::Subject,
        cur_y, sw,
    );
    cur_y += CHAR_H * 0.9;

    // Body label + editor
    draw_text_ex("  Message:", 8.0, cur_y,
        TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });
    cur_y += CHAR_H * 0.9;

    let editor_x = 12.0;
    let editor_y = cur_y;
    let editor_w = sw - 24.0;
    let editor_h = footer_y - editor_y - 8.0;
    let active   = compose.field == ComposeMailField::Body;

    draw_rectangle_lines(editor_x - 2.0, editor_y, editor_w + 4.0, editor_h, 1.0,
        if active { GREEN_BBS } else { DARKGRAY });

    let rows_vis  = ((editor_h - 8.0) / CHAR_H) as usize;
    let body_lines: Vec<&str> = compose.body.split('\n').collect();
    let total     = body_lines.len();
    let scroll    = total.saturating_sub(rows_vis);

    for (i, line) in body_lines[scroll..].iter().enumerate() {
        let y = editor_y + 6.0 + (i as f32 + 1.0) * CHAR_H;
        if y > footer_y - 8.0 { break; }
        draw_text_ex(line, editor_x + 4.0, y,
            TextParams { font_size: FONT_SIZE, color: WHITE_BBS, ..Default::default() });
    }

    if active {
        let last = body_lines.last().copied().unwrap_or("");
        let vis_row = total - 1 - scroll;
        let cy = editor_y + 6.0 + (vis_row as f32 + 1.0) * CHAR_H;
        if cy < footer_y - 8.0 {
            let cx = editor_x + 4.0 + last.len() as f32 * CHAR_W;
            draw_rectangle(cx, cy - CHAR_H + 4.0, CHAR_W, CHAR_H - 2.0,
                Color::new(0.0, 0.75, 0.0, 0.8));
        }
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    draw_hint_bar(footer_y + CHAR_H * 1.2, &[
        ("[Tab]", " Next Field  "),
        ("[F1]", " Send  "),
        ("[Esc]", " Abort"),
    ]);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn draw_field(label: &str, value: &str, active: bool, y: f32, sw: f32) -> f32 {
    let lw = measure_text(label, None, FONT_SIZE, 1.0).width;
    draw_text_ex(label, 8.0, y,
        TextParams { font_size: FONT_SIZE, color: CYAN_BBS, ..Default::default() });

    let fx = 8.0 + lw;
    let fw = sw - fx - 16.0;
    draw_rectangle_lines(fx - 2.0, y - CHAR_H + 2.0, fw, CHAR_H + 4.0, 1.0,
        if active { GREEN_BBS } else { DARKGRAY });
    draw_text_ex(value, fx + 2.0, y,
        TextParams { font_size: FONT_SIZE, color: GREEN_BBS, ..Default::default() });

    if active {
        let cx = fx + 2.0 + value.len() as f32 * CHAR_W;
        draw_rectangle(cx, y - CHAR_H + 4.0, CHAR_W, CHAR_H - 2.0,
            Color::new(0.0, 0.75, 0.0, 0.8));
    }
    y + CHAR_H * 1.3
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}

fn draw_hint_bar(y: f32, hints: &[(&str, &str)]) {
    let mut hx = 8.0;
    for (key, desc) in hints {
        draw_text_ex(key, hx, y,
            TextParams { font_size: FONT_SIZE, color: YELLOW, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, y,
            TextParams { font_size: FONT_SIZE, color: DARKGRAY, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}
