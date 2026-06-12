use macroquad::prelude::*;

use crate::app::{App, ComposeField};
use crate::tui::font::{ch, cw, s, txt, tw};

pub fn render_compose(app: &App) {
    let compose = match app.compose.as_ref() {
        Some(c) => c,
        None => return,
    };

    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = if compose.thread_id.is_none() {
        format!("  Compose: New Thread in {}", compose.board_name)
    } else {
        format!("  Compose: Reply to \"{}\"", clip(&compose.reply_subject, 55))
    };
    txt(&title, s(8.0), ch() * 1.3, t.title);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    let mut cur_y = body_y + ch() * 1.3;

    if compose.thread_id.is_none() {
        let label = "  Subject : ";
        let lw = tw(label);
        txt(label, s(8.0), cur_y, t.label);
        let fx = s(8.0) + lw;
        let fw = sw - fx - s(16.0);
        let active_subj = compose.field == ComposeField::Subject;
        draw_rectangle_lines(fx - s(2.0), cur_y - ch() + s(2.0), fw, ch() + s(4.0), s(1.0),
            if active_subj { t.hi } else { t.border });
        txt(&compose.subject, fx + s(2.0), cur_y, t.hi);
        if active_subj {
            let cx = fx + s(2.0) + tw(&compose.subject);
            draw_rectangle(cx, cur_y - ch() + s(4.0), cw(), ch() - s(2.0), t.cursor);
        }
        cur_y += ch() * 1.8;
    } else {
        txt(
            &format!("  Re: {}", clip(&compose.reply_subject, 70)),
            s(8.0), cur_y,
            t.lo,
        );
        cur_y += ch() * 1.6;
    }

    txt("  Message:", s(8.0), cur_y, t.label);
    cur_y += ch() * 0.9;

    let editor_x = s(12.0);
    let editor_y = cur_y;
    let editor_w = sw - s(24.0);
    let editor_h = footer_y - editor_y - s(8.0);
    let active_body = compose.field == ComposeField::Body;

    draw_rectangle_lines(editor_x - s(2.0), editor_y, editor_w + s(4.0), editor_h, s(1.0),
        if active_body { t.hi } else { t.border });

    let rows_vis = ((editor_h - s(8.0)) / ch()) as usize;
    let body_lines: Vec<&str> = compose.body.split('\n').collect();
    let total  = body_lines.len();
    let scroll = total.saturating_sub(rows_vis);

    for (i, line) in body_lines[scroll..].iter().enumerate() {
        let y = editor_y + s(6.0) + (i as f32 + 1.0) * ch();
        if y > footer_y - s(8.0) { break; }
        txt(line, editor_x + s(4.0), y, t.body);
    }

    if active_body {
        let last_line = body_lines.last().copied().unwrap_or("");
        let vis_row   = total - 1 - scroll;
        let cy        = editor_y + s(6.0) + (vis_row as f32 + 1.0) * ch();
        if cy < footer_y - s(8.0) {
            let cx = editor_x + s(4.0) + tw(last_line);
            draw_rectangle(cx, cy - ch() + s(4.0), cw(), ch() - s(2.0), t.cursor);
        }
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    let hints: &[(&str, &str)] = if compose.thread_id.is_none() {
        &[("[Tab]", " Switch Field  "), ("[F1]", " Post  "), ("[Esc]", " Abort")]
    } else {
        &[("[F1]", " Post Reply  "), ("[Esc]", " Abort")]
    };
    let mut hx = s(8.0);
    let hy = footer_y + ch() * 1.2;
    for (key, desc) in hints {
        txt(key, hx, hy, t.title);
        hx += tw(key);
        txt(desc, hx, hy, t.border);
        hx += tw(desc);
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}
