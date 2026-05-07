use macroquad::prelude::*;

use crate::app::App;
use crate::tui::theme::Theme;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

const POLL_H_VOTED:   f32 = CHAR_H * 3.8;
const POLL_H_UNVOTED: f32 = CHAR_H * 2.8;

pub fn render_voting(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!(
        "  {}  --  VOTING BOOTH  ({} poll{})",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.polls.len(),
        if app.polls.len() == 1 { "" } else { "s" },
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Body ─────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    if app.polls.is_empty() {
        draw_text_ex("  No polls available.", 12.0, body_y + CHAR_H * 2.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    let mut py = body_y + CHAR_H * 0.8;
    let bar_max_w = sw - 140.0;

    for (idx, poll) in app.polls.iter().enumerate() {
        if idx < app.voting_scroll { continue; }

        let voted = app.voted_polls.contains(&poll.id);
        let item_h = if voted { POLL_H_VOTED } else { POLL_H_UNVOTED };
        if py + item_h > footer_y { break; }

        let selected = idx == app.selected_poll_row;

        if selected {
            draw_rectangle(2.0, py - 2.0, sw - 4.0, item_h - 4.0, t.sel_bg);
        }

        let q_color = if selected { t.title } else { t.primary };
        let prefix  = if selected { "►" } else { " " };
        let num_label = format!("  {} {}. ", prefix, idx + 1);
        let nl_w = measure_text(&num_label, None, FONT_SIZE, 1.0).width;
        draw_text_ex(&num_label, 8.0, py + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: if selected { t.highlight } else { t.dim }, ..Default::default() });
        draw_text_ex(&poll.question, 8.0 + nl_w, py + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: q_color, ..Default::default() });

        if voted {
            let total = (poll.yes_votes + poll.no_votes).max(1) as f64;
            let yes_pct = poll.yes_votes as f64 / total;
            let no_pct  = poll.no_votes  as f64 / total;

            draw_bar(8.0, py + CHAR_H * 2.1, bar_max_w, "YES", yes_pct, poll.yes_votes, t.primary, t);
            draw_bar(8.0, py + CHAR_H * 3.1, bar_max_w, "NO ", no_pct,  poll.no_votes,  t.dim,    t);
        } else {
            let prompt_color = if selected { t.secondary } else { t.dim };
            draw_text_ex(
                "       Press [Y]es or [N]o to cast your vote",
                8.0, py + CHAR_H * 2.1,
                TextParams { font_size: FONT_SIZE, color: prompt_color, ..Default::default() },
            );
        }

        py += item_h;
    }

    if app.polls.len() > 3 {
        let max_idx = app.polls.len().saturating_sub(1);
        let pct = if max_idx == 0 { 100u32 } else {
            (app.selected_poll_row as f32 / max_idx as f32 * 100.0) as u32
        };
        draw_text_ex(&format!("{:3}%", pct), sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    // ── Footer ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);

    let selected_voted = app.polls.get(app.selected_poll_row)
        .map(|p| app.voted_polls.contains(&p.id))
        .unwrap_or(true);

    let mut hx = 8.0;
    let vote_color = if selected_voted { t.dim } else { t.title };
    let hints: &[(&str, &str, Color)] = &[
        ("[↑↓/jk]", " Navigate  ", t.title),
        ("[Y]",      " Yes  ",      vote_color),
        ("[N]",      " No  ",       vote_color),
        ("[Esc]",    " Main Menu",  t.title),
    ];
    for (key, desc, kc) in hints {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: *kc, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

fn draw_bar(x: f32, y: f32, bar_max_w: f32, label: &str, pct: f64, votes: u32,
            bar_color: Color, t: &Theme) {
    let label_text = format!("       {} ", label);
    let lw = measure_text(&label_text, None, FONT_SIZE, 1.0).width;

    draw_text_ex(&label_text, x, y,
        TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });

    let bar_x = x + lw;
    let bar_w = bar_max_w - lw;
    let bar_h = CHAR_H * 0.65;
    let filled = (bar_w * pct as f32).max(0.0);

    draw_rectangle(bar_x, y - bar_h + 2.0, filled, bar_h, bar_color);
    draw_rectangle_lines(bar_x, y - bar_h + 2.0, bar_w, bar_h, 1.0, t.dim);

    let stat = format!("  {:>5} ({:3.0}%)", votes, pct * 100.0);
    draw_text_ex(&stat, bar_x + bar_w + 4.0, y,
        TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
}
