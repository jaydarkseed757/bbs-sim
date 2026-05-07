use macroquad::prelude::*;

use crate::app::App;

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;

// ── File list ─────────────────────────────────────────────────────────────────

pub fn render_file_list(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let title = format!(
        "  {}  --  FILES",
        app.session.bbs_name.as_deref().unwrap_or("BBS"),
    );
    draw_text_ex(&title, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);
    draw_text_ex(
        "   NAME                     SIZE    DATE       DESCRIPTION",
        12.0, body_y + CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() },
    );
    draw_line(4.0, body_y + CHAR_H * 1.25, sw - 4.0, body_y + CHAR_H * 1.25, 1.0, t.border);

    let flat: Vec<_> = app.files.iter()
        .flat_map(|s| s.files.iter())
        .collect();

    let mut row_y = body_y + CHAR_H * 2.2;
    let mut flat_idx: usize = 0;

    for section in &app.files {
        if row_y + CHAR_H > footer_y { break; }

        draw_text_ex(
            &format!("  ── {}  ", section.name),
            12.0, row_y,
            TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() },
        );
        let lw = measure_text(&format!("  ── {}  ", section.name), None, FONT_SIZE, 1.0).width;
        draw_line(12.0 + lw, row_y - CHAR_H * 0.35, sw - 12.0, row_y - CHAR_H * 0.35, 1.0, t.dim);
        row_y += CHAR_H * 1.4;

        for file in &section.files {
            if row_y + CHAR_H > footer_y { break; }

            let selected = flat_idx == app.selected_file_row;
            if selected {
                draw_rectangle(2.0, row_y - CHAR_H + 4.0, sw - 4.0, CHAR_H + 2.0, t.sel_bg);
            }

            let fg     = if selected { t.primary } else { t.secondary };
            let prefix = if selected { "►" } else { " " };
            let line = format!(
                "  {} {:<22}  {:<6}  {:<10}  {}",
                prefix,
                clip(&file.name, 22),
                clip(&file.size, 6),
                clip(&file.date, 10),
                clip(&file.description, 45),
            );
            draw_text_ex(&line, 12.0, row_y,
                TextParams { font_size: FONT_SIZE, color: fg, ..Default::default() });

            flat_idx += 1;
            row_y += CHAR_H * 1.5;
        }

        row_y += CHAR_H * 0.3;
    }

    if flat.is_empty() {
        draw_text_ex(
            "  No files.",
            12.0, body_y + CHAR_H * 3.0,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() },
        );
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);

    let selected_file = flat.get(app.selected_file_row);
    let can_view = selected_file.map(|f| f.kind == "text").unwrap_or(false);
    let view_color = if can_view { t.title } else { t.dim };

    let mut hx = 8.0;
    let hints: &[(&str, &str, Color)] = &[
        ("[↑↓/jk]", " Navigate  ", t.title),
        ("[V]",      " View  ",     view_color),
        ("[D]",      " Download  ", t.title),
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

// ── View file ─────────────────────────────────────────────────────────────────

pub fn render_view_file(app: &App, id: u32) {
    let t = &app.theme;
    let file = match app.files.iter()
        .flat_map(|s| s.files.iter())
        .find(|f| f.id == id)
    {
        Some(f) => f,
        None    => return,
    };

    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 3.0;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    draw_text_ex(&format!("  {}", file.name), 8.0, CHAR_H * 1.0,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
    draw_text_ex(&format!("  Size: {}   Date: {}", file.size, file.date),
        8.0, CHAR_H * 2.1,
        TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    let lines: Vec<&str> = file.content.split('\n').collect();
    let rows_vis   = ((body_h - 16.0) / CHAR_H) as usize;
    let max_scroll = lines.len().saturating_sub(rows_vis);
    let scroll     = app.file_view_scroll.min(max_scroll);

    for (i, line) in lines[scroll..lines.len().min(scroll + rows_vis)].iter().enumerate() {
        let y = body_y + 8.0 + (i as f32 + 1.0) * CHAR_H;
        draw_text_ex(line, 12.0, y,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
    }

    if lines.len() > rows_vis {
        let pct = if max_scroll == 0 { 100 } else { (scroll as f32 / max_scroll as f32 * 100.0) as u32 };
        draw_text_ex(&format!("{:3}%", pct), sw - 55.0, body_y + CHAR_H,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    let mut hx = 8.0;
    for (key, desc) in &[("[↑↓]", " Scroll  "), ("[PgUp/PgDn]", " Page  "), ("[Esc]", " Back")] {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}~", t)
}
