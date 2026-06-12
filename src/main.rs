use macroquad::prelude::*;
use macroquad::texture::FilterMode;
use macroquad::audio::load_sound_from_bytes;

mod app;
mod bbs;
mod save;
mod sim;
mod tui;

use app::{App, Sounds};
use sim::audio::{carrier_drop_wav, connect_wav, handshake_wav, nav_click_wav, no_answer_wav};
use tui::font::set_bbs_font;

fn window_conf() -> Conf {
    Conf {
        window_title: "BBS-SIM".to_owned(),
        window_width: 1024,
        window_height: 768,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_default_filter_mode(FilterMode::Nearest);

    // Load Monaco for a crisp monospace terminal look.
    // Falls back to the built-in font if the file is unavailable.
    if let Ok(font) = load_ttf_font("/System/Library/Fonts/Monaco.ttf").await {
        set_bbs_font(font);
    }

    let mut app = App::new();

    // Pre-generate and load all sound effects.
    let hs  = load_sound_from_bytes(&handshake_wav()).await.ok();
    let con = load_sound_from_bytes(&connect_wav()).await.ok();
    let na  = load_sound_from_bytes(&no_answer_wav()).await.ok();
    let cd  = load_sound_from_bytes(&carrier_drop_wav()).await.ok();
    let nc  = load_sound_from_bytes(&nav_click_wav()).await.ok();

    if let (Some(handshake), Some(connect), Some(no_answer), Some(carrier_drop), Some(nav_click))
        = (hs, con, na, cd, nc)
    {
        app.sounds = Some(Sounds { handshake, connect, no_answer, carrier_drop, nav_click });
    }

    let mut tick_accum = 0.0_f32;

    loop {
        clear_background(BLACK);

        app.handle_input();

        tick_accum += get_frame_time();
        if tick_accum >= 0.05 {
            app.tick();
            tick_accum -= 0.05;
        }

        app.render();

        if app.should_quit {
            break;
        }

        next_frame().await;
    }

    app.print_exit_stats();
}
