use macroquad::prelude::*;

mod app;
mod bbs;
mod sim;
mod tui;

use app::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "BBS-SIM".to_owned(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
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
}
