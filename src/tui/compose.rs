use macroquad::prelude::*;

use crate::app::{App, ComposeField};

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const CHAR_W: f32 = FONT_SIZE as f32 * 0.55;

pub fn render_compose(app: &App) {
    let t       = &app.theme;
    let compose = match app.compose.as_ref() {
        Some(c) => c,
        None => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = if compose.thread_id.is_none() {
        format!("  Compose: New Thread in {}", compose.board_name)
    } else {
        format!("  Compose: Reply to \"{}\"", clip(&compose.reply_subject, 55))
    };
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Form area ─────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    let mut cur_y = body_y + CHAR_H * 1.3;

    if compose.thread_id.is_none() {
        let label = "  Subject : ";
        let lw = measure_text(label, None, FONT_SIZE, 1.0).width;
        draw_text_ex(label, 8.0, cur_y,
            TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });

        let fx = 8.0 + lw;
        let fw = sw - fx - 16.0;
        let active_subj = compose.field == ComposeField::Subject;
        draw_rectangle_lines(fx - 2.0, cur_y - CHAR_H + 2.0, fw, CHAR_H + 4.0, 1.0,
            if active_subj { t.primary } else { t.border });
        draw_text_ex(&compose.subject, fx + 2.0, cur_y,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });

        if active_subj {
            let cx = fx + 2.0 + compose.subject.len() as f32 * CHAR_W;
            draw_rectangle(cx, cur_y - CHAR_H + 4.0, CHAR_W, CHAR_H - 2.0, t.cursor);
        }
        cur_y += CHAR_H * 1.8;
    } else {
        draw_text_ex(
            &format!("  Re: {}", clip(&compose.reply_subject, 70)),
            8.0, cur_y,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
        cur_y += CHAR_H * 1.6;
    }

    draw_text_ex("  Message:", 8.0, cur_y,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    cur_y += CHAR_H * 0.9;

    let editor_x = 12.0;
    let editor_y = cur_y;
    let editor_w = sw - 24.0;
    let editor_h = footer_y - editor_y - 8.0;
    let active_body = compose.field == ComposeField::Body;

    draw_rectangle_lines(editor_x - 2.0, editor_y, editor_w + 4.0, editor_h, 1.0,
        if active_body { t.primary } else { t.border });

    let rows_vis = ((editor_h - 8.0) / CHAR_H) as usize;
    let body_lines: Vec<&str> = compose.body.split('\n').collect();
    let total   = body_lines.len();
    let scroll  = total.saturating_sub(rows_vis);

    for (i, line) in body_lines[scroll..].iter().enumerate() {
        let y = editor_y + 6.0 + (i as f32 + 1.0) * CHAR_H;
        if y > footer_y - 8.0 { break; }
        draw_text_ex(line, editor_x + 4.0, y,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
    }

    if active_body {
        let last_line = body_lines.last().copied().unwrap_or("");
        let vis_row   = total - 1 - scroll;
        let cy        = editor_y + 6.0 + (vis_row as f32 + 1.0) * CHAR_H;
        if cy < footer_y - 8.0 {
            let cx = editor_x + 4.0 + last_line.len() as f32 * CHAR_W;
            draw_rectangle(cx, cy - CHAR_H + 4.0, CHAR_W, CHAR_H - 2.0, t.cursor);
        }
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    let hints: &[(&str, &str)] = if compose.thread_id.is_none() {
        &[("[Tab]", " Switch Field  "), ("[F1]", " Post  "), ("[Esc]", " Abort")]
    } else {
        &[("[F1]", " Post Reply  "), ("[Esc]", " Abort")]
    };
    let mut hx = 8.0;
    let hy = footer_y + CHAR_H * 1.2;
    for (key, desc) in hints {
        draw_text_ex(key, hx, hy,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, hy,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}
