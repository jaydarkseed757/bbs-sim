use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod bbs;
mod event;
mod sim;
mod tui;

use app::App;
use event::{EventHandler, InputEvent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut events = EventHandler::new();

    let result = run(&mut terminal, &mut app, &mut events).await;

    // Always restore the terminal, even on error.
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(std::time::Duration::from_millis(50));

    loop {
        terminal.draw(|f| app.render(f))?;

        tokio::select! {
            maybe = events.next() => {
                if let Some(ev) = maybe {
                    match ev {
                        InputEvent::Key(key) => app.handle_input(key),
                        InputEvent::Resize(_, _) => {}
                    }
                }
            }
            _ = ticker.tick() => {
                app.tick();
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
