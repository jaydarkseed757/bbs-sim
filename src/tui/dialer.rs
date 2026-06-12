use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, cw, fs, s, txt, txt_sz, tw};

const CYAN: Color = Color::new(0.0, 0.87, 0.87, 1.0);

const HEADER_ART: &[&str] = &[
    r"  ____  ____  ____       ____  ___ __  __ ",
    r" | __ )| __ )/ ___|     / ___|_ _|  \/  |",
    r" |  _ \|  _ \\___ \ ____\___ \| || |\/| |",
    r" | |_) | |_) |___) |____|__) | || |  | |",
    r" |____/|____/|____/     |____/___|_|  |_|",
    r"   Bulletin Board System Simulator v0.1   ",
];

const HEADER_ROWS: usize = 6;

pub fn render_dialer(app: &App) {
    let sw = screen_width();
    let sh = screen_height();

    let header_h = ch() * (HEADER_ROWS as f32 + 2.0);
    let footer_h = ch() * 2.5;
    let table_y  = header_h;
    let footer_y = sh - footer_h;

    // ── Header zone ──────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);

    let title = "  BBS-SIM v0.1.0 -- PHONEBOOK DIALER  ";
    txt(title, s(8.0), ch(), YELLOW);

    // ASCII art — pulsing brightness with a phosphor glow halo
    let t = get_time() as f32;
    let pulse = 0.78 + 0.18 * (t * 1.1).sin().abs();
    let logo_color = Color::new(0.05, pulse, 0.05, 1.0);
    let glow_color = Color::new(0.0, pulse * 0.45, 0.0, 0.10);

    for (i, line) in HEADER_ART.iter().enumerate() {
        let y = ch() * (i as f32 + 2.0);
        for &(ox, oy) in &[(-1.0f32, 0.0f32), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            txt_sz(line, s(8.0) + ox, y + oy, fs(), glow_color);
        }
        txt(line, s(8.0), y, logo_color);
    }

    // ── Table zone ───────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), table_y, sw - s(2.0), footer_y - table_y - s(1.0), s(1.5), DARKGRAY);
    txt(" PHONEBOOK ", s(8.0), table_y + ch() * 0.8, CYAN);

    let col_pct = [0.42_f32, 0.22, 0.12, 0.24];
    let col_x: Vec<f32> = col_pct
        .iter()
        .scan(s(4.0), |acc, &p| {
            let x = *acc;
            *acc += sw * p;
            Some(x)
        })
        .collect();

    let header_y = table_y + ch() * 1.8;
    let col_labels = ["  BBS NAME", "PHONE NUMBER", "BAUD", "LAST CALLED"];
    for (i, label) in col_labels.iter().enumerate() {
        txt(label, col_x[i], header_y, YELLOW);
    }

    draw_line(s(4.0), header_y + s(4.0), sw - s(4.0), header_y + s(4.0), s(1.0), DARKGRAY);

    let row_start_y = header_y + ch() * 1.4;
    for (i, entry) in app.phonebook.iter().enumerate() {
        let ry = row_start_y + i as f32 * ch() * 1.25;
        if ry + ch() > footer_y { break; }

        let selected = i == app.selected_row;
        let (fg, prefix) = if selected {
            draw_rectangle(s(2.0), ry - ch() + s(4.0), sw - s(4.0), ch() + s(2.0), Color::new(0.0, 0.25, 0.0, 1.0));
            (GREEN, "►  ")
        } else {
            (Color::new(0.0, 0.65, 0.0, 1.0), "   ")
        };

        let last = entry.last_called.as_deref().unwrap_or("Never");
        let cells = [
            format!("{}{}", prefix, entry.name),
            entry.number.clone(),
            entry.baud.to_string(),
            last.to_string(),
        ];
        for (j, cell) in cells.iter().enumerate() {
            txt(cell, col_x[j], ry, fg);
        }
    }

    // ── Footer / help bar ────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);

    let hints: &[(&str, &str)] = &[
        ("[↑↓/jk]", " Navigate  "),
        ("[D]", " Dial  "),
        ("[M]", " Manual Dial  "),
        ("[Q/Esc]", " Quit"),
    ];
    let mut hx = s(8.0);
    let hy = footer_y + ch() * 1.2;
    for (key, desc) in hints {
        txt(key, hx, hy, YELLOW);
        hx += tw(key);
        txt(desc, hx, hy, DARKGRAY);
        hx += tw(desc);
    }

    // ── Manual dial overlay ───────────────────────────────────────────────────
    if let Some(ref number) = app.manual_dial_input {
        let box_w = sw * 0.42;
        let box_h = ch() * 5.0;
        let box_x = (sw - box_w) * 0.5;
        let box_y = (sh - box_h) * 0.5;

        draw_rectangle(box_x, box_y, box_w, box_h, Color::new(0.0, 0.0, 0.0, 0.92));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, s(1.5), CYAN);

        txt("  Manual Dial", box_x + s(8.0), box_y + ch() * 1.1, YELLOW);

        let label = "  Number: ";
        txt(label, box_x + s(8.0), box_y + ch() * 2.4, CYAN);
        let lw = tw(label);
        txt(number, box_x + s(8.0) + lw, box_y + ch() * 2.4, GREEN);

        // Cursor
        let nw = tw(number);
        draw_rectangle(
            box_x + s(8.0) + lw + nw, box_y + ch() * 2.4 - ch() + s(4.0),
            cw(), ch() - s(2.0),
            Color::new(0.0, 0.75, 0.0, 0.8),
        );

        txt("  [Enter] Dial   [Esc] Cancel", box_x + s(8.0), box_y + ch() * 3.8, DARKGRAY);
    }
}
