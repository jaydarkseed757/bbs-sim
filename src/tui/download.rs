use macroquad::prelude::*;

use crate::app::{App, DownloadPhase};
use crate::tui::font::{ch, s, txt, tw};

const GREEN_BBS: Color  = Color::new(0.0,  0.85, 0.0,  1.0);
const DIM_GREEN: Color  = Color::new(0.0,  0.55, 0.0,  1.0);
const CYAN_BBS:  Color  = Color::new(0.0,  0.87, 0.87, 1.0);
const DIM_GRAY:  Color  = Color::new(0.35, 0.35, 0.35, 1.0);
const AMBER:     Color  = Color::new(1.0,  0.75, 0.0,  1.0);

pub fn render_download(app: &App) {
    let dl = match app.download.as_ref() {
        Some(d) => d,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(
        &format!("  {}  --  FILE TRANSFER", dl.bbs_name),
        s(8.0), ch() * 1.3,
        YELLOW,
    );

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let lx  = s(16.0);
    let vx  = s(185.0);   // wide enough for "  Transferred :" at 18px Monaco
    let mut cy = body_y + ch() * 1.4;

    // File info block
    let label_color = CYAN_BBS;
    txt("  File     :", lx, cy, label_color);
    txt(&dl.file_name, vx, cy, GREEN_BBS);
    cy += ch() * 1.4;

    txt("  Size     :", lx, cy, label_color);
    txt(
        &format!("{} bytes  ({})", fmt_bytes(dl.file_size_bytes), dl.file_size_str),
        vx, cy, DIM_GREEN,
    );
    cy += ch() * 1.4;

    txt("  Protocol :", lx, cy, label_color);
    txt("ZMODEM", vx, cy, GREEN_BBS);
    cy += ch() * 1.4;

    txt("  Link     :", lx, cy, label_color);
    txt(&format!("{} baud", dl.baud), vx, cy, DIM_GREEN);
    cy += ch() * 2.0;

    // ── Progress bar ─────────────────────────────────────────────────────────
    let bar_x  = s(16.0);
    let bar_w  = sw - s(32.0);
    let pct = if dl.file_size_bytes == 0 { 1.0_f32 }
              else { dl.bytes_done as f32 / dl.file_size_bytes as f32 };
    let pct_clamped = pct.min(1.0);

    // Bar background
    draw_rectangle(bar_x, cy - ch() * 0.8, bar_w, ch() * 1.1, Color::new(0.05, 0.05, 0.05, 1.0));
    draw_rectangle_lines(bar_x, cy - ch() * 0.8, bar_w, ch() * 1.1, s(1.0), DIM_GRAY);

    // Filled portion
    let fill_w = (bar_w * pct_clamped).max(0.0);
    let bar_color = if pct_clamped >= 1.0 { GREEN_BBS } else { Color::new(0.0, 0.65, 0.0, 1.0) };
    draw_rectangle(bar_x, cy - ch() * 0.8, fill_w, ch() * 1.1, bar_color);

    // Percentage label centered on the bar
    let pct_str = format!("{:3.0}%", pct_clamped * 100.0);
    let pct_w   = tw(&pct_str);
    let pct_x   = bar_x + (bar_w - pct_w) * 0.5;
    // Draw label in inverted color so it's visible over both filled and unfilled
    let label_txt_color = if pct_clamped >= 1.0 { BLACK } else { GREEN_BBS };
    txt(&pct_str, pct_x, cy, label_txt_color);
    cy += ch() * 1.8;

    // ── Transfer stats ───────────────────────────────────────────────────────
    match &dl.phase {
        DownloadPhase::Negotiating { .. } => {
            txt("  Negotiating protocol...", lx, cy, AMBER);
            cy += ch() * 1.4;
        }
        DownloadPhase::Transferring => {
            txt("  Transferred :", lx, cy, label_color);
            txt(&format!("{} bytes", fmt_bytes(dl.bytes_done)), vx, cy, GREEN_BBS);
            cy += ch() * 1.4;

            txt("  Remaining   :", lx, cy, label_color);
            let remaining = dl.file_size_bytes.saturating_sub(dl.bytes_done);
            txt(&format!("{} bytes", fmt_bytes(remaining)), vx, cy, DIM_GREEN);
            cy += ch() * 1.4;

            txt("  Speed       :", lx, cy, label_color);
            txt(&format!("{} CPS", fmt_cps(dl.cps_display)), vx, cy, AMBER);
            cy += ch() * 1.4;

            txt("  Elapsed     :", lx, cy, label_color);
            txt(&fmt_time(dl.elapsed_secs()), vx, cy, DIM_GREEN);
            cy += ch() * 1.4;

            txt("  ETA         :", lx, cy, label_color);
            let eta = if dl.cps_display > 0.0 {
                remaining as f32 / dl.cps_display
            } else { 0.0 };
            txt(&fmt_time(eta), vx, cy, DIM_GREEN);
            cy += ch() * 1.8;
        }
        DownloadPhase::Complete { elapsed_secs } => {
            let avg_cps = if *elapsed_secs > 0.0 {
                dl.file_size_bytes as f32 / elapsed_secs
            } else { dl.cps_display };

            txt("  Transferred :", lx, cy, label_color);
            txt(&format!("{} bytes", fmt_bytes(dl.file_size_bytes)), vx, cy, GREEN_BBS);
            cy += ch() * 1.4;

            txt("  Elapsed     :", lx, cy, label_color);
            txt(&fmt_time(*elapsed_secs), vx, cy, DIM_GREEN);
            cy += ch() * 1.4;

            txt("  Avg speed   :", lx, cy, label_color);
            txt(&format!("{} CPS", fmt_cps(avg_cps)), vx, cy, AMBER);
            cy += ch() * 2.0;

            txt("  Transfer complete!", lx, cy, GREEN_BBS);
            cy += ch() * 0.4;
        }
    }

    // ── ZMODEM protocol log ───────────────────────────────────────────────────
    let log_y     = cy;
    let log_h     = footer_y - log_y - s(8.0);
    let rows_vis  = ((log_h - s(4.0)) / ch()) as usize;

    if rows_vis > 0 {
        draw_rectangle_lines(s(8.0), log_y, sw - s(16.0), log_h, s(1.0), DIM_GRAY);

        let lines = &dl.log_lines;
        let start = lines.len().saturating_sub(rows_vis);
        for (i, line) in lines[start..].iter().enumerate() {
            let ly = log_y + s(4.0) + (i as f32 + 1.0) * ch();
            if ly > footer_y - s(8.0) { break; }
            // Color: protocol keywords vs data noise
            let lc = if line.starts_with("ZR") || line.starts_with("ZF") || line.starts_with("ZD")
                        || line.starts_with("ZE") || line.starts_with("Transfer")
                        || line.starts_with("Receiving") || line.starts_with("CONNECT")
                     { GREEN_BBS }
                     else { DIM_GRAY };
            txt(&format!("  {}", line), s(8.0), ly, lc);
        }
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    let (hint_key, hint_desc) = match &dl.phase {
        DownloadPhase::Complete { .. } => ("[Any key]", " Return to files"),
        _                              => ("[Esc]", " Cancel transfer"),
    };
    let mut hx = s(8.0);
    txt(hint_key, hx, footer_y + ch() * 1.2, YELLOW);
    hx += tw(hint_key);
    txt(hint_desc, hx, footer_y + ch() * 1.2, DARKGRAY);
}

// ── Formatting helpers ────────────────────────────────────────────────────────

fn fmt_bytes(b: u64) -> String {
    // Insert commas every 3 digits
    let s = b.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn fmt_cps(cps: f32) -> String {
    fmt_bytes(cps as u64)
}

fn fmt_time(secs: f32) -> String {
    let s  = secs as u32;
    let m  = s / 60;
    let ss = s % 60;
    if m >= 60 {
        let h = m / 60;
        let mm = m % 60;
        format!("{}:{:02}:{:02}", h, mm, ss)
    } else {
        format!("{}:{:02}", m, ss)
    }
}
