use macroquad::prelude::*;

use crate::bbs::data::BbsEntry;
use crate::sim::modem::DialPhase;
use crate::tui::dialer::render_dialer;

// ---------------------------------------------------------------------------
// Screen state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Screen {
    Dialer,
    Dialing { bbs: BbsEntry, phase: DialPhase },
    Login { bbs: BbsEntry },
    MainMenu,
    MessageBoards,
    ReadThread { board_id: String, thread_id: u32 },
    ComposeMessage,
    Files,
    Logout,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub phonebook: Vec<BbsEntry>,
    pub selected_row: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dialer,
            should_quit: false,
            phonebook: hardcoded_phonebook(),
            selected_row: 0,
        }
    }

    pub fn render(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer => render_dialer(self),
            _ => render_stub(),
        }
    }

    pub fn handle_input(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer => self.dialer_input(),
            _ => {
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Dialer;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        // Drive animations, dial phases, baud-rate output — stub for now.
    }

    // ── Dialer input ────────────────────────────────────────────────────────

    fn dialer_input(&mut self) {
        let len = self.phonebook.len();

        // Arrow keys (edge-triggered)
        if is_key_pressed(KeyCode::Up) {
            self.selected_row = if self.selected_row == 0 { len.saturating_sub(1) } else { self.selected_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_row = if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
        }
        if is_key_pressed(KeyCode::Escape) {
            self.should_quit = true;
        }

        // Character keys
        while let Some(ch) = get_char_pressed() {
            match ch {
                'q' | 'Q' => self.should_quit = true,
                'j' => {
                    self.selected_row = if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
                }
                'k' => {
                    self.selected_row = if self.selected_row == 0 { len.saturating_sub(1) } else { self.selected_row - 1 };
                }
                'd' | 'D' => {
                    if let Some(bbs) = self.phonebook.get(self.selected_row).cloned() {
                        self.screen = Screen::Dialing {
                            bbs,
                            phase: DialPhase::PickingUpLine,
                        };
                    }
                }
                _ => {}
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
