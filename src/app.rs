use macroquad::prelude::*;

use crate::bbs::data::BbsEntry;
use crate::sim::modem::{DialPhase, ModemSim};
use crate::sim::typer::BaudTyper;
use crate::tui::dialer::render_dialer;
use crate::tui::dialing::render_dialing;
use crate::tui::terminal::{CellStyle, TerminalBuffer};

// ---------------------------------------------------------------------------
// Screen state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Screen {
    Dialer,
    Dialing,
    Login { bbs: BbsEntry },
    MainMenu,
    MessageBoards,
    ReadThread { board_id: String, thread_id: u32 },
    ComposeMessage,
    Files,
    Logout,
}

// ---------------------------------------------------------------------------
// DialingState — live state while the modem handshake plays out
// ---------------------------------------------------------------------------

pub struct DialingState {
    pub bbs: BbsEntry,
    pub modem: ModemSim,
    pub typer: BaudTyper,
    pub buffer: TerminalBuffer,
    pub awaiting_keypress: bool,
}

impl DialingState {
    fn new(bbs: BbsEntry) -> Self {
        let modem = ModemSim::new(&bbs.number);
        let typer = BaudTyper::new(bbs.baud);
        let buffer = TerminalBuffer::new(80, 500);
        Self { bbs, modem, typer, buffer, awaiting_keypress: false }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub phonebook: Vec<BbsEntry>,
    pub selected_row: usize,
    pub dial: Option<DialingState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dialer,
            should_quit: false,
            phonebook: hardcoded_phonebook(),
            selected_row: 0,
            dial: None,
        }
    }

    pub fn render(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer  => render_dialer(self),
            Screen::Dialing => render_dialing(self),
            _               => render_stub(),
        }
    }

    pub fn handle_input(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer  => self.dialer_input(),
            Screen::Dialing => self.dialing_input(),
            _ => {
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Dialer;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        if !matches!(self.screen, Screen::Dialing) {
            return;
        }

        let Some(ref mut d) = self.dial else { return };

        // 1. Advance modem one step; enqueue any emitted text.
        if let Some(text) = d.modem.tick() {
            d.typer.enqueue(&text);
        }

        // 2. Drain chars from the baud typer into the terminal buffer.
        let style = CellStyle::fg(Color::new(0.0, 0.85, 0.0, 1.0));
        for ch in d.typer.tick() {
            d.buffer.push_char(ch, style);
        }

        // 3. Once the modem is fully connected and the queue is drained,
        //    print the login prompt and wait for a keypress.
        if d.modem.phase == DialPhase::Connected
            && d.typer.is_empty()
            && !d.awaiting_keypress
        {
            d.buffer.push_str(
                "\r\n\r\nPress [Enter] to log in...\r\n",
                CellStyle::fg(YELLOW),
            );
            d.awaiting_keypress = true;
        }
    }

    // ── Dialer input ────────────────────────────────────────────────────────

    fn dialer_input(&mut self) {
        let len = self.phonebook.len();

        if is_key_pressed(KeyCode::Up) {
            self.selected_row =
                if self.selected_row == 0 { len.saturating_sub(1) } else { self.selected_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_row =
                if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
        }
        if is_key_pressed(KeyCode::Escape) {
            self.should_quit = true;
        }

        while let Some(ch) = get_char_pressed() {
            match ch {
                'q' | 'Q' => self.should_quit = true,
                'j' => {
                    self.selected_row =
                        if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
                }
                'k' => {
                    self.selected_row =
                        if self.selected_row == 0 { len.saturating_sub(1) } else { self.selected_row - 1 };
                }
                'd' | 'D' => {
                    if let Some(bbs) = self.phonebook.get(self.selected_row).cloned() {
                        self.dial = Some(DialingState::new(bbs));
                        self.screen = Screen::Dialing;
                    }
                }
                _ => {}
            }
        }
    }

    // ── Dialing input ────────────────────────────────────────────────────────

    fn dialing_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Dialer;
            self.dial = None;
            return;
        }

        let awaiting = self.dial.as_ref().map(|d| d.awaiting_keypress).unwrap_or(false);
        if awaiting && is_key_pressed(KeyCode::Enter) {
            if let Some(d) = &self.dial {
                self.screen = Screen::Login { bbs: d.bbs.clone() };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Placeholder for unimplemented screens
// ---------------------------------------------------------------------------

fn render_stub() {
    draw_rectangle_lines(0.0, 0.0, screen_width(), screen_height(), 2.0, DARKGRAY);
    draw_text(
        "Screen not yet implemented -- press [Esc] to return.",
        16.0,
        32.0,
        20.0,
        WHITE,
    );
}

// ---------------------------------------------------------------------------
// Hardcoded sample phonebook
// ---------------------------------------------------------------------------

fn hardcoded_phonebook() -> Vec<BbsEntry> {
    vec![
        BbsEntry {
            name: "The Rusty Nail BBS".into(),
            number: "555-6502".into(),
            sysop: "ShadowByte".into(),
            location: "Cincinnati, OH".into(),
            baud: 2400,
            boards: vec!["General".into(), "Warez".into(), "C64".into(), "Tech Talk".into()],
            last_called: Some("04/22/93".into()),
        },
        BbsEntry {
            name: "WARP FACTOR 9".into(),
            number: "555-1701".into(),
            sysop: "KirkFan".into(),
            location: "Seattle, WA".into(),
            baud: 9600,
            boards: vec!["SciFi".into(), "Gaming".into(), "Anime".into()],
            last_called: Some("04/15/93".into()),
        },
        BbsEntry {
            name: "The Digital Dungeon".into(),
            number: "555-4096".into(),
            sysop: "DungeonMstr".into(),
            location: "Austin, TX".into(),
            baud: 2400,
            boards: vec!["RPG".into(), "D&D".into(), "General".into()],
            last_called: None,
        },
        BbsEntry {
            name: "ELITE FORCE BBS".into(),
            number: "555-3133".into(),
            sysop: "31337".into(),
            location: "Boston, MA".into(),
            baud: 14400,
            boards: vec!["Warez".into(), "Hacking".into(), "Music".into()],
            last_called: Some("04/01/93".into()),
        },
        BbsEntry {
            name: "The Underground Railroad".into(),
            number: "555-7777".into(),
            sysop: "ConductorX".into(),
            location: "Chicago, IL".into(),
            baud: 2400,
            boards: vec!["Politics".into(), "News".into(), "General".into()],
            last_called: Some("03/28/93".into()),
        },
        BbsEntry {
            name: "Midnight Rendezvous".into(),
            number: "555-2600".into(),
            sysop: "Pr0phet".into(),
            location: "New York, NY".into(),
            baud: 9600,
            boards: vec!["Phreaking".into(), "Anarchy".into(), "Carding".into()],
            last_called: Some("03/14/93".into()),
        },
    ]
}
