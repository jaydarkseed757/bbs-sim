use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const GREEN: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const YELLOW_BBS: Color = Color::new(1.0, 0.85, 0.0, 1.0);
const DIM_GRAY: Color = Color::new(0.35, 0.35, 0.35, 1.0);

pub fn render_download(app: &App) {
    let dl = match app.download.as_ref() {
        Some(d) => d,
        None => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y = header_h;
    let body_h = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);
    let bbs_label = app.session.bbs_name.as_deref().unwrap_or("BBS");
    draw_text_ex(
        &format!("  ZMODEM TRANSFER — {}", bbs_label),
        8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: YELLOW_BBS, ..Default::default() },
    );

    // ── Body ─────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, DARKGRAY);
    let lx = 12.0;
    let mut ly = body_y + CHAR_H * 2.0;

    draw_text_ex(
        &format!("  Receiving: {}", dl.file_name),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );
    ly += CHAR_H;
    draw_text_ex(
        &format!("  File size: {} bytes", fmt_num(dl.total_bytes)),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );
    ly += CHAR_H * 1.8;

    // Progress bar
    let bar_x = lx + 10.0;
    let bar_w = sw - bar_x - 80.0;
    let bar_h = CHAR_H * 0.8;
    let pct = if dl.total_bytes > 0 {
        dl.transferred as f64 / dl.total_bytes as f64
    } else {
        0.0
    };
    draw_rectangle(bar_x, ly, (bar_w * pct as f32).max(0.0), bar_h, GREEN);
    draw_rectangle_lines(bar_x, ly, bar_w, bar_h, 1.5, DIM_GRAY);
    let pct_label = format!("{:3.0}%", pct * 100.0);
    draw_text_ex(
        &pct_label,
        bar_x + bar_w + 8.0,
        ly + bar_h * 0.85,
        TextParams { font_size: FONT_SIZE, color: YELLOW_BBS, ..Default::default() },
    );
    ly += CHAR_H * 2.0;

    // Stats
    let elapsed_secs = dl.elapsed_ticks / 20;
    let remaining_secs = if pct > 0.001 {
        let total_ticks_est = dl.elapsed_ticks as f64 / pct;
        let remaining_ticks = total_ticks_est - dl.elapsed_ticks as f64;
        (remaining_ticks / 20.0) as u64
    } else {
        0
    };

    draw_text_ex(
        &format!("  Bytes recv:     {} of {}", fmt_num(dl.transferred), fmt_num(dl.total_bytes)),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );
    ly += CHAR_H;
    draw_text_ex(
        &format!("  Transfer rate:  {} bps", fmt_num(dl.baud as u64)),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );
    ly += CHAR_H;
    draw_text_ex(
        &format!("  Elapsed:        {:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );
    ly += CHAR_H;
    if !dl.done {
        draw_text_ex(
            &format!("  Remaining:      {:02}:{:02}", remaining_secs / 60, remaining_secs % 60),
            lx, ly,
            TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
        );
    }
    ly += CHAR_H;
    draw_text_ex(
        &format!("  Errors: 0   Retries: 0   Packets: {}", fmt_num(dl.packet_count)),
        lx, ly,
        TextParams { font_size: FONT_SIZE, color: GREEN, ..Default::default() },
    );

    if dl.done {
        ly += CHAR_H * 1.8;
        draw_text_ex(
            "  *** Transfer complete!  CRC OK. ***",
            lx, ly,
            TextParams { font_size: FONT_SIZE, color: YELLOW_BBS, ..Default::default() },
        );
    }

    // ── Footer ───────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);
    let (hint, color) = if dl.done {
        ("  [Enter] Continue", YELLOW_BBS)
    } else {
        ("  [Esc] Abort", DIM_GRAY)
    };
    draw_text_ex(
        hint,
        8.0, footer_y + CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color, ..Default::default() },
    );
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}
