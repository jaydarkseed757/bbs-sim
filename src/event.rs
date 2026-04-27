use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum InputEvent {
    Key(KeyEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<InputEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<InputEvent>();

        // Crossterm's poll/read are blocking; run them on a dedicated OS thread
        // so the tokio runtime is never stalled.
        std::thread::spawn(move || loop {
            match event::poll(Duration::from_millis(50)) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(k)) => {
                        if tx.send(InputEvent::Key(k)).is_err() {
                            break;
                        }
                    }
                    Ok(Event::Resize(w, h)) => {
                        if tx.send(InputEvent::Resize(w, h)).is_err() {
                            break;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<InputEvent> {
        self.rx.recv().await
    }
}
