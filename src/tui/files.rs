use macroquad::prelude::*;

use crate::app::App;
use crate::tui::font::{ch, s, txt, tw};

const GREEN_BBS: Color = Color::new(0.0, 0.85, 0.0, 1.0);
const DIM_GREEN: Color = Color::new(0.0, 0.55, 0.0, 1.0);
const WHITE_BBS: Color = Color::new(0.85, 0.85, 0.85, 1.0);
const CYAN_BBS:  Color = Color::new(0.0, 0.87, 0.87, 1.0);
const DIM_GRAY:  Color = Color::new(0.35, 0.35, 0.35, 1.0);

// ── File list ─────────────────────────────────────────────────────────────────

pub fn render_file_list(app: &App) {
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(
        &format!("  {}  --  FILES", app.session.bbs_name.as_deref().unwrap_or("BBS")),
        s(8.0), ch() * 1.3,
        YELLOW,
    );

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);
    txt(
        "   NAME                     SIZE    DATE       DESCRIPTION",
        s(12.0), body_y + ch() * 1.0,
        CYAN_BBS,
    );
    draw_line(s(4.0), body_y + ch() * 1.25, sw - s(4.0), body_y + ch() * 1.25, s(1.0), DARKGRAY);

    let flat: Vec<_> = app.files.iter().flat_map(|s| s.files.iter()).collect();

    let mut row_y = body_y + ch() * 2.2;
    let mut flat_idx: usize = 0;

    for section in &app.files {
        if row_y + ch() > footer_y { break; }

        let section_label = format!("  ── {}  ", section.name);
        txt(&section_label, s(12.0), row_y, CYAN_BBS);
        let lw = tw(&section_label);
        draw_line(s(12.0) + lw, row_y - ch() * 0.35, sw - s(12.0), row_y - ch() * 0.35, s(1.0), DIM_GRAY);
        row_y += ch() * 1.4;

        for file in &section.files {
            if row_y + ch() > footer_y { break; }

            let selected = flat_idx == app.selected_file_row;
            if selected {
                draw_rectangle(s(2.0), row_y - ch() + s(4.0), sw - s(4.0), ch() + s(2.0),
                    Color::new(0.0, 0.25, 0.0, 1.0));
            }

            let fg     = if selected { GREEN_BBS } else { DIM_GREEN };
            let prefix = if selected { "►" } else { " " };
            let line = format!(
                "  {} {:<22}  {:<6}  {:<10}  {}",
                prefix,
                clip(&file.name, 22),
                clip(&file.size, 6),
                clip(&file.date, 10),
                clip(&file.description, 45),
            );
            txt(&line, s(12.0), row_y, fg);

            flat_idx += 1;
            row_y += ch() * 1.5;
        }

        row_y += ch() * 0.3;
    }

    if flat.is_empty() {
        txt("  No files.", s(12.0), body_y + ch() * 3.0, DIM_GRAY);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);

    let selected_file = flat.get(app.selected_file_row);
    let can_view = selected_file.map(|f| f.kind == "text").unwrap_or(false);
    let view_color = if can_view { YELLOW } else { DIM_GRAY };
    let mut hx = s(8.0);
    let hints: &[(&str, &str, Color)] = &[
        ("[↑↓/jk]", " Navigate  ", YELLOW),
        ("[V]",     " View  ",      view_color),
        ("[D]",     " Download  ", YELLOW),
        ("[Esc]",   " Main Menu",  YELLOW),
    ];
    for (key, desc, kc) in hints {
        txt(key, hx, footer_y + ch() * 1.2, *kc);
        hx += tw(key);
        txt(desc, hx, footer_y + ch() * 1.2, DARKGRAY);
        hx += tw(desc);
    }
}

// ── View file ─────────────────────────────────────────────────────────────────

pub fn render_view_file(app: &App, id: u32) {
    let file = match app.files.iter()
        .flat_map(|s| s.files.iter())
        .find(|f| f.id == id)
    {
        Some(f) => f,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 3.0;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), DARKGRAY);
    txt(&format!("  {}", file.name), s(8.0), ch() * 1.0, YELLOW);
    txt(&format!("  Size: {}   Date: {}", file.size, file.date), s(8.0), ch() * 2.1, DIM_GREEN);

    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), DARKGRAY);

    let lines: Vec<&str> = file.content.split('\n').collect();
    let rows_vis   = ((body_h - s(16.0)) / ch()) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.file_view_scroll.min(max_scroll);

    for (i, line) in lines[scroll..lines.len().min(scroll + rows_vis)].iter().enumerate() {
        let y = body_y + s(8.0) + (i as f32 + 1.0) * ch();
        txt(line, s(12.0), y, WHITE_BBS);
    }

    if lines.len() > rows_vis {
        let pct = if max_scroll == 0 { 100 } else { (scroll as f32 / max_scroll as f32 * 100.0) as u32 };
        txt(&format!("{:3}%", pct), sw - s(55.0), body_y + ch(), DIM_GRAY);
    }

    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), DARKGRAY);
    draw_hint_bar(footer_y + ch() * 1.2, &[
        ("[↑↓]", " Scroll  "),
        ("[PgUp/PgDn]", " Page  "),
        ("[Esc]", " Back"),
    ]);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
