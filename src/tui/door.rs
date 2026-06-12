use macroquad::prelude::*;

use crate::app::{App, DoorGameState, DoorPhase};
use crate::tui::font::{ch, s, txt, txt_sz, tw};
use crate::tui::theme::BbsTheme;

pub fn render_door_game(app: &App) {
    let t  = &app.theme;
    let sw = screen_width();
    let sh = screen_height();
    let header_h = ch() * 2.2;
    let footer_h = ch() * 2.2;
    let body_y   = header_h;
    let body_h   = sh - header_h - footer_h;
    let footer_y = sh - footer_h;

    let Some(ref g) = app.door_game else { return };

    // ── Header ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), s(1.0), sw - s(2.0), header_h - s(1.0), s(1.5), t.border);
    let subtitle = match &g.phase {
        DoorPhase::ClassSelect  => "  THE GAUNTLET  --  Choose your class".to_string(),
        DoorPhase::Combat       => format!("  THE GAUNTLET  --  Round {}  |  Score: {}", g.round, g.score),
        DoorPhase::BetweenRounds=> format!("  THE GAUNTLET  --  Round {} complete!", g.round),
        DoorPhase::Dead         => "  THE GAUNTLET  --  GAME OVER".to_string(),
    };
    txt(&subtitle, s(8.0), ch() * 1.3, t.title);

    // ── Body ──────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), body_y, sw - s(2.0), body_h, s(1.5), t.border);

    match &g.phase {
        DoorPhase::ClassSelect => render_class_select(body_y, body_h, sw, t),
        DoorPhase::Combat | DoorPhase::BetweenRounds => render_combat(g, body_y, body_h, sw, t),
        DoorPhase::Dead => render_dead(g, body_y, body_h, sw, t),
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    draw_rectangle_lines(s(1.0), footer_y, sw - s(2.0), footer_h - s(1.0), s(1.5), t.border);
    let hints: &[(&str, &str)] = match &g.phase {
        DoorPhase::ClassSelect   => &[("[1-3]", " Choose class  "), ("[Esc]", " Exit")],
        DoorPhase::Combat        => &[("[A]", " Attack  "), ("[Esc]", " Flee")],
        DoorPhase::BetweenRounds => &[("[Any key]", " Next round  "), ("[Esc]", " Exit")],
        DoorPhase::Dead          => &[("[R]", " Play again  "), ("[Esc]", " Exit")],
    };
    let mut hx = s(8.0);
    for (key, desc) in hints {
        txt(key, hx, footer_y + ch() * 1.2, t.title);
        hx += tw(key);
        txt(desc, hx, footer_y + ch() * 1.2, t.border);
        hx += tw(desc);
    }
}

fn render_class_select(body_y: f32, _body_h: f32, _sw: f32, t: &BbsTheme) {
    let mut y = body_y + ch() * 1.8;

    txt("  Welcome to The Gauntlet! Fight your way through increasingly", s(12.0), y, t.lo);
    y += ch() * 1.4;
    txt("  dangerous foes. How many rounds can you survive?", s(12.0), y, t.lo);
    y += ch() * 2.2;

    txt("  Choose your class:", s(12.0), y, t.label);
    y += ch() * 1.8;

    let classes: &[(&str, &str, &str, &str)] = &[
        ("[1]", "WARRIOR", "HP: 30   ATK:  3-8 ", "Stalwart defender. High health, steady damage."),
        ("[2]", "MAGE",    "HP: 20   ATK:  6-14", "Glass cannon. Low health, devastating spells."),
        ("[3]", "ROGUE",   "HP: 25   ATK:  4-11", "Balanced fighter. Reliable all-rounder."),
    ];

    for (key, name, stats, desc) in classes {
        txt(key, s(28.0), y, t.title);
        let kw = tw(key);
        txt(&format!(" {}", name), s(28.0) + kw, y, t.hi);
        txt(stats, s(180.0), y, t.label);
        y += ch() * 1.2;
        txt(&format!("      {}", desc), s(28.0), y, t.muted);
        y += ch() * 1.8;
    }
}

fn render_combat(g: &DoorGameState, body_y: f32, _body_h: f32, sw: f32, t: &BbsTheme) {
    let mut y = body_y + ch() * 1.4;

    // ── Combatant info row ────────────────────────────────────────────────────
    let class_name = g.class.as_ref().map(|c| c.name()).unwrap_or("???");
    let max_hp     = g.player_max_hp();

    txt(&format!("  {}", class_name), s(12.0), y, t.hi);

    if let Some(ref e) = g.enemy {
        let ex = sw - s(200.0);
        txt(&e.name, ex, y, t.label);
    }
    y += ch() * 1.3;

    // Player HP bar
    let bar_w = s(180.0);
    draw_hp_bar(s(12.0), y, bar_w, g.hp, max_hp, t.hi, t);
    if let Some(ref e) = g.enemy {
        draw_hp_bar(sw - s(200.0), y, bar_w, e.hp, e.max_hp, t.label, t);
    }
    y += ch() * 1.8;

    // ── Divider ───────────────────────────────────────────────────────────────
    draw_line(s(4.0), y, sw - s(4.0), y, s(1.0), t.border);
    y += ch() * 0.8;

    // ── Combat log ───────────────────────────────────────────────────────────
    for line in &g.log {
        txt(line, s(16.0), y, t.lo);
        y += ch() * 1.3;
    }

    // ── Between-rounds overlay ────────────────────────────────────────────────
    if g.phase == DoorPhase::BetweenRounds {
        y += ch() * 0.6;
        txt(
            &format!("  Enemy defeated! Score: {}   Press any key for round {}...",
                g.score, g.round + 1),
            s(12.0), y,
            t.title,
        );
    }
}

fn render_dead(g: &DoorGameState, body_y: f32, _body_h: f32, _sw: f32, t: &BbsTheme) {
    let class_name = g.class.as_ref().map(|c| c.name()).unwrap_or("???");
    let mut y = body_y + ch() * 2.0;

    txt_sz("  YOU HAVE BEEN SLAIN!", s(12.0), y, crate::tui::font::fs() + 4, t.title);
    y += ch() * 2.2;

    let rows: &[(&str, String)] = &[
        ("  Class    :", class_name.to_string()),
        ("  Slain by :", g.slain_by.clone()),
        ("  Rounds   :", format!("{}", g.round)),
        ("  Score    :", format!("{} pts", g.score)),
    ];
    for (label, val) in rows {
        txt(label, s(12.0), y, t.label);
        let lw = tw(label);
        txt(&format!(" {}", val), s(12.0) + lw, y, t.hi);
        y += ch() * 1.6;
    }
}

fn draw_hp_bar(x: f32, y: f32, w: f32, hp: i32, max_hp: i32, bar_color: Color, t: &BbsTheme) {
    let hp_clamped = hp.max(0);
    let pct = hp_clamped as f32 / max_hp.max(1) as f32;
    let bar_h = ch() * 0.6;
    let filled = w * pct;

    draw_rectangle(x, y - bar_h, filled, bar_h, bar_color);
    draw_rectangle_lines(x, y - bar_h, w, bar_h, s(1.0), t.muted);

    let label = format!("HP {}/{}", hp_clamped, max_hp);
    let lw = tw(&label);
    txt(&label, x + w / 2.0 - lw / 2.0, y, t.muted);
}
