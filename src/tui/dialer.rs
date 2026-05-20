use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: f32 = 18.0;
const CYAN: Color = Color::new(0.0, 0.87, 0.87, 1.0);
const CHAR_H: f32 = FONT_SIZE * 1.3;

// TODO: replace with a TTF font that covers box-drawing (U+2500-U+257F) for
//       authentic borders. The built-in macroquad bitmap font is ASCII-only.
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

    let header_h = CHAR_H * (HEADER_ROWS as f32 + 2.0); // art + padding
    let footer_h = CHAR_H * 2.5;
    let table_y = header_h;
    let footer_y = sh - footer_h;

    // ── Header zone ──────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, DARKGRAY);

    // Title line
    let title = "  BBS-SIM v0.1.0 -- PHONEBOOK DIALER  ";
    draw_text_ex(
        title,
        8.0,
        CHAR_H,
        TextParams { font_size: FONT_SIZE as u16, color: YELLOW, ..Default::default() },
    );

    // ASCII art — pulsing brightness with a phosphor glow halo
    let t = get_time() as f32;
    let pulse = 0.78 + 0.18 * (t * 1.1).sin().abs();
    let logo_color = Color::new(0.05, pulse, 0.05, 1.0);
    let glow_color = Color::new(0.0, pulse * 0.45, 0.0, 0.10);

    for (i, line) in HEADER_ART.iter().enumerate() {
        let y = CHAR_H * (i as f32 + 2.0);
        // Glow halo (4 offset passes)
        for &(ox, oy) in &[(-1.0f32, 0.0f32), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            draw_text_ex(
                line, 8.0 + ox, y + oy,
                TextParams { font_size: FONT_SIZE as u16, color: glow_color, ..Default::default() },
            );
        }
        draw_text_ex(
            line, 8.0, y,
            TextParams { font_size: FONT_SIZE as u16, color: logo_color, ..Default::default() },
        );
    }

    // ── Table zone ───────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, table_y, sw - 2.0, footer_y - table_y - 1.0, 1.5, DARKGRAY);
    draw_text_ex(
        " PHONEBOOK ",
        8.0,
        table_y + CHAR_H * 0.8,
        TextParams { font_size: FONT_SIZE as u16, color: CYAN, ..Default::default() },
    );

    // Column layout — cumulative x offsets matching percentage widths
    let col_pct = [0.42_f32, 0.22, 0.12, 0.24];
    let col_x: Vec<f32> = col_pct
        .iter()
        .scan(4.0_f32, |acc, &p| {
            let x = *acc;
            *acc += sw * p;
            Some(x)
        })
        .collect();

    // Column headers
    let header_y = table_y + CHAR_H * 1.8;
    let col_labels = ["  BBS NAME", "PHONE NUMBER", "BAUD", "LAST CALLED"];
    for (i, label) in col_labels.iter().enumerate() {
        draw_text_ex(
            label,
            col_x[i],
            header_y,
            TextParams { font_size: FONT_SIZE as u16, color: YELLOW, ..Default::default() },
        );
    }

    // Divider under headers
    draw_line(4.0, header_y + 4.0, sw - 4.0, header_y + 4.0, 1.0, DARKGRAY);

    // Data rows
    let row_start_y = header_y + CHAR_H * 1.4;
    for (i, entry) in app.phonebook.iter().enumerate() {
        let ry = row_start_y + i as f32 * CHAR_H * 1.25;

        // Clip rows that would overflow into footer
        if ry + CHAR_H > footer_y {
            break;
        }

        let selected = i == app.selected_row;
        let (fg, prefix) = if selected {
            draw_rectangle(2.0, ry - CHAR_H + 4.0, sw - 4.0, CHAR_H + 2.0, Color::new(0.0, 0.25, 0.0, 1.0));
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
            draw_text_ex(
                cell,
                col_x[j],
                ry,
                TextParams { font_size: FONT_SIZE as u16, color: fg, ..Default::default() },
            );
        }
    }

    // ── Footer / help bar ────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, DARKGRAY);

    let hints: &[(&str, &str)] = &[
        ("[↑↓/jk]", " Navigate  "),
        ("[D]", " Dial  "),
        ("[M]", " Manual Dial  "),
        ("[Q/Esc]", " Quit"),
    ];

    let mut hx = 8.0;
    let hy = footer_y + CHAR_H * 1.2;
    for (key, desc) in hints {
        draw_text_ex(
            key,
            hx,
            hy,
            TextParams { font_size: FONT_SIZE as u16, color: YELLOW, ..Default::default() },
        );
        hx += measure_text(key, None, FONT_SIZE as u16, 1.0).width;
        draw_text_ex(
            desc,
            hx,
            hy,
            TextParams { font_size: FONT_SIZE as u16, color: DARKGRAY, ..Default::default() },
        );
        hx += measure_text(desc, None, FONT_SIZE as u16, 1.0).width;
    }

    // ── Manual dial overlay ───────────────────────────────────────────────────
    if let Some(ref number) = app.manual_dial_input {
        let box_w = sw * 0.42;
        let box_h = CHAR_H * 5.0;
        let box_x = (sw - box_w) * 0.5;
        let box_y = (sh - box_h) * 0.5;

        draw_rectangle(box_x, box_y, box_w, box_h, Color::new(0.0, 0.0, 0.0, 0.92));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 1.5, CYAN);

        draw_text_ex(
            "  Manual Dial",
            box_x + 8.0, box_y + CHAR_H * 1.1,
            TextParams { font_size: FONT_SIZE as u16, color: YELLOW, ..Default::default() },
        );

        let label = "  Number: ";
        draw_text_ex(
            label,
            box_x + 8.0, box_y + CHAR_H * 2.4,
            TextParams { font_size: FONT_SIZE as u16, color: CYAN, ..Default::default() },
        );
        let lw = measure_text(label, None, FONT_SIZE as u16, 1.0).width;
        draw_text_ex(
            number,
            box_x + 8.0 + lw, box_y + CHAR_H * 2.4,
            TextParams { font_size: FONT_SIZE as u16, color: GREEN, ..Default::default() },
        );
        // Cursor
        let nw = measure_text(number, None, FONT_SIZE as u16, 1.0).width;
        let cw = FONT_SIZE * 0.55;
        draw_rectangle(
            box_x + 8.0 + lw + nw, box_y + CHAR_H * 2.4 - CHAR_H + 4.0,
            cw, CHAR_H - 2.0,
            Color::new(0.0, 0.75, 0.0, 0.8),
        );

        draw_text_ex(
            "  [Enter] Dial   [Esc] Cancel",
            box_x + 8.0, box_y + CHAR_H * 3.8,
            TextParams { font_size: FONT_SIZE as u16, color: DARKGRAY, ..Default::default() },
        );
    }
}
