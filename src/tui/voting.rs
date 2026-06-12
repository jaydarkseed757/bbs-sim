use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt, tw};
use crate::tui::theme::BbsTheme;

pub fn render_voting(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // Item heights — mirror VOTE_H_VOTED / VOTE_H_UNVOTED in app.rs.
    let poll_h_voted   = ch() * 3.8;
    let poll_h_unvoted = ch() * 2.8;

    // ── Header ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let title = format!(
        "  {}  --  VOTING BOOTH  ({} poll{})",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
        app.polls.len(),
        if app.polls.len() == 1 { "" } else { "s" },
    );
    txt(&title, s(8.0), ch() * 1.3, t.title);

    // ── Body ─────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    if app.polls.is_empty() {
        txt("  No polls available.", s(12.0), body_y + ch() * 2.0, t.muted);
    }

    let mut py = body_y + ch() * 0.8;
    let bar_max_w = sw - s(140.0);

    for (idx, poll) in app.polls.iter().enumerate() {
        if idx < app.voting_scroll { continue; }

        let voted = app.voted_polls.contains(&poll.id);
        let item_h = if voted { poll_h_voted } else { poll_h_unvoted };
        if py + item_h > footer_y { break; }

        let selected = idx == app.selected_poll_row;

        if selected {
            draw_rectangle(s(2.0), py - s(2.0), sw - s(4.0), item_h - s(4.0), t.sel_bg);
        }

        let q_color = if selected { t.title } else { t.hi };
        let prefix  = if selected { "►" } else { " " };
        let num_label = format!("  {} {}. ", prefix, idx + 1);
        let nl_w = tw(&num_label);
        txt(&num_label, s(8.0), py + ch(), if selected { t.label } else { t.muted });
        txt(&poll.question, s(8.0) + nl_w, py + ch(), q_color);

        if voted {
            let total = (poll.yes_votes + poll.no_votes).max(1) as f64;
            let yes_pct = poll.yes_votes as f64 / total;
            let no_pct  = poll.no_votes  as f64 / total;

            draw_bar(s(8.0), py + ch() * 2.1, bar_max_w, "YES", yes_pct, poll.yes_votes, t.hi,    t);
            draw_bar(s(8.0), py + ch() * 3.1, bar_max_w, "NO ", no_pct,  poll.no_votes,  t.muted, t);
        } else {
            let prompt_color = if selected { t.lo } else { t.muted };
            txt(
                "       Press [Y]es or [N]o to cast your vote",
                s(8.0), py + ch() * 2.1,
                prompt_color,
            );
        }

        py += item_h;
    }

    if app.polls.len() > 3 {
        let max_idx = app.polls.len().saturating_sub(1);
        let pct = if max_idx == 0 { 100u32 } else {
            (app.selected_poll_row as f32 / max_idx as f32 * 100.0) as u32
        };
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), t.muted);
    }

    // ── Footer ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);

    let selected_voted = app.polls.get(app.selected_poll_row)
        .map(|p| app.voted_polls.contains(&p.id))
        .unwrap_or(true);

    let mut hx = s(8.0);
    let vote_color = if selected_voted { t.muted } else { t.title };
    let hints: &[(&str, &str, Color)] = &[
        ("[↑↓/jk]", " Navigate  ", t.title),
        ("[Y]",      " Yes  ",      vote_color),
        ("[N]",      " No  ",       vote_color),
        ("[Esc]",    " Main Menu",  t.title),
    ];
    for (key, desc, kc) in hints {
        txt(key, hx, footer_y + ch() * 1.2, *kc);
        hx += tw(key);
        txt(desc, hx, footer_y + ch() * 1.2, t.border);
        hx += tw(desc);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bar(x: f32, y: f32, bar_max_w: f32, label: &str, pct: f64, votes: u32,
            bar_color: Color, t: &BbsTheme) {
    let label_text = format!("       {} ", label);
    let lw = tw(&label_text);

    txt(&label_text, x, y, t.muted);

    let bar_x = x + lw;
    let bar_w = bar_max_w - lw;
    let bar_h = ch() * 0.65;
    let filled = (bar_w * pct as f32).max(0.0);

    draw_rectangle(bar_x, y - bar_h + s(2.0), filled, bar_h, bar_color);
    draw_rectangle_lines(bar_x, y - bar_h + s(2.0), bar_w, bar_h, s(1.0), t.muted);

    let stat = format!("  {:>5} ({:3.0}%)", votes, pct * 100.0);
    txt(&stat, bar_x + bar_w + s(4.0), y, t.lo);
}
