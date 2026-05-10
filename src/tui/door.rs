use macroquad::prelude::*;

use crate::app::{App, DoorPhase};

const FONT_SIZE: u16 = 18;
const CHAR_H: f32 = FONT_SIZE as f32 * 1.35;
const CHAR_W: f32 = FONT_SIZE as f32 * 0.55;

pub fn render_door_game(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = CHAR_H * 2.2;
    let footer_h = CHAR_H * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let Some(ref g) = app.door_game else { return };

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, 1.0, sw - 2.0, header_h - 1.0, 1.5, t.border);
    let subtitle = match &g.phase {
        DoorPhase::ClassSelect  => "  THE GAUNTLET  --  Choose your class".to_string(),
        DoorPhase::Combat       => format!("  THE GAUNTLET  --  Round {}  |  Score: {}", g.round, g.score),
        DoorPhase::BetweenRounds=> format!("  THE GAUNTLET  --  Round {} complete!", g.round),
        DoorPhase::Dead         => "  THE GAUNTLET  --  GAME OVER".to_string(),
    };
    draw_text_ex(&subtitle, 8.0, CHAR_H * 1.3,
        TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, body_y, sw - 2.0, body_h, 1.5, t.border);

    match &g.phase {
        DoorPhase::ClassSelect => render_class_select(body_y, body_h, sw, t),
        DoorPhase::Combat | DoorPhase::BetweenRounds => render_combat(g, body_y, body_h, sw, t),
        DoorPhase::Dead => render_dead(g, body_y, body_h, sw, t),
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(1.0, footer_y, sw - 2.0, footer_h - 1.0, 1.5, t.border);
    let hints: &[(&str, &str)] = match &g.phase {
        DoorPhase::ClassSelect   => &[("[1-3]", " Choose class  "), ("[Esc]", " Exit")],
        DoorPhase::Combat        => &[("[A]", " Attack  "), ("[Esc]", " Flee")],
        DoorPhase::BetweenRounds => &[("[Any key]", " Next round  "), ("[Esc]", " Exit")],
        DoorPhase::Dead          => &[("[R]", " Play again  "), ("[Esc]", " Exit")],
    };
    let mut hx = 8.0;
    for (key, desc) in hints {
        draw_text_ex(key, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        hx += measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(desc, hx, footer_y + CHAR_H * 1.2,
            TextParams { font_size: FONT_SIZE, color: t.border, ..Default::default() });
        hx += measure_text(desc, None, FONT_SIZE, 1.0).width;
    }
}

fn render_class_select(body_y: f32, _body_h: f32, _sw: f32, t: &crate::tui::theme::Theme) {
    let mut y = body_y + CHAR_H * 1.8;

    draw_text_ex("  Welcome to The Gauntlet! Fight your way through increasingly", 12.0, y,
        TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
    y += CHAR_H * 1.4;
    draw_text_ex("  dangerous foes. How many rounds can you survive?", 12.0, y,
        TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
    y += CHAR_H * 2.2;

    draw_text_ex("  Choose your class:", 12.0, y,
        TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    y += CHAR_H * 1.8;

    let classes: &[(&str, &str, &str, &str)] = &[
        ("[1]", "WARRIOR", "HP: 30   ATK:  3-8 ", "Stalwart defender. High health, steady damage."),
        ("[2]", "MAGE",    "HP: 20   ATK:  6-14", "Glass cannon. Low health, devastating spells."),
        ("[3]", "ROGUE",   "HP: 25   ATK:  4-11", "Balanced fighter. Reliable all-rounder."),
    ];

    for (key, name, stats, desc) in classes {
        draw_text_ex(key, 28.0, y,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() });
        let kw = measure_text(key, None, FONT_SIZE, 1.0).width;
        draw_text_ex(&format!(" {}", name), 28.0 + kw, y,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        draw_text_ex(stats, 180.0, y,
            TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
        y += CHAR_H * 1.2;
        draw_text_ex(&format!("      {}", desc), 28.0, y,
            TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
        y += CHAR_H * 1.8;
    }
}

fn render_combat(g: &crate::app::DoorGameState, body_y: f32, _body_h: f32, sw: f32,
                 t: &crate::tui::theme::Theme) {
    let mut y = body_y + CHAR_H * 1.4;

    // ── Combatant info row ────────────────────────────────────────────────────
    let class_name = g.class.as_ref().map(|c| c.name()).unwrap_or("???");
    let max_hp     = g.player_max_hp();

    draw_text_ex(&format!("  {}", class_name), 12.0, y,
        TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });

    if let Some(ref e) = g.enemy {
        let ex = sw - 200.0;
        draw_text_ex(&e.name, ex, y,
            TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
    }
    y += CHAR_H * 1.3;

    // Player HP bar
    let bar_w = 180.0;
    draw_hp_bar(12.0, y, bar_w, g.hp, max_hp, t.primary, t);
    if let Some(ref e) = g.enemy {
        draw_hp_bar(sw - 200.0, y, bar_w, e.hp, e.max_hp, t.highlight, t);
    }
    y += CHAR_H * 1.8;

    // ── Divider ───────────────────────────────────────────────────────────────
    draw_line(4.0, y, sw - 4.0, y, 1.0, t.border);
    y += CHAR_H * 0.8;

    // ── Combat log ───────────────────────────────────────────────────────────
    for line in &g.log {
        draw_text_ex(line, 16.0, y,
            TextParams { font_size: FONT_SIZE, color: t.secondary, ..Default::default() });
        y += CHAR_H * 1.3;
    }

    // ── Between-rounds overlay ────────────────────────────────────────────────
    if g.phase == DoorPhase::BetweenRounds {
        y += CHAR_H * 0.6;
        draw_text_ex(
            &format!("  Enemy defeated! Score: {}   Press any key for round {}...",
                g.score, g.round + 1),
            12.0, y,
            TextParams { font_size: FONT_SIZE, color: t.title, ..Default::default() },
        );
    }
}

fn render_dead(g: &crate::app::DoorGameState, body_y: f32, _body_h: f32, _sw: f32,
               t: &crate::tui::theme::Theme) {
    let class_name = g.class.as_ref().map(|c| c.name()).unwrap_or("???");
    let mut y = body_y + CHAR_H * 2.0;

    draw_text_ex("  YOU HAVE BEEN SLAIN!", 12.0, y,
        TextParams { font_size: FONT_SIZE + 4, color: t.title, ..Default::default() });
    y += CHAR_H * 2.2;

    let rows: &[(&str, String)] = &[
        ("  Class    :", class_name.to_string()),
        ("  Slain by :", g.slain_by.clone()),
        ("  Rounds   :", format!("{}", g.round)),
        ("  Score    :", format!("{} pts", g.score)),
    ];
    for (label, val) in rows {
        draw_text_ex(label, 12.0, y,
            TextParams { font_size: FONT_SIZE, color: t.highlight, ..Default::default() });
        let lw = measure_text(label, None, FONT_SIZE, 1.0).width;
        draw_text_ex(&format!(" {}", val), 12.0 + lw, y,
            TextParams { font_size: FONT_SIZE, color: t.primary, ..Default::default() });
        y += CHAR_H * 1.6;
    }
}

fn draw_hp_bar(x: f32, y: f32, w: f32, hp: i32, max_hp: i32,
               bar_color: Color, t: &crate::tui::theme::Theme) {
    let hp_clamped = hp.max(0);
    let pct = hp_clamped as f32 / max_hp as f32;
    let bar_h = CHAR_H * 0.6;
    let filled = w * pct;

    draw_rectangle(x, y - bar_h, filled, bar_h, bar_color);
    draw_rectangle_lines(x, y - bar_h, w, bar_h, 1.0, t.dim);

    let label = format!("HP {}/{}", hp_clamped, max_hp);
    let lw = measure_text(&label, None, FONT_SIZE, 1.0).width;
    draw_text_ex(&label, x + w / 2.0 - lw / 2.0, y,
        TextParams { font_size: FONT_SIZE, color: t.dim, ..Default::default() });
    let _ = CHAR_W; // used via FONT_SIZE ratio
}
