use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{widgets::TableState, Frame};

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
    pub dialer_state: TableState,
}

impl App {
    pub fn new() -> Self {
        let mut dialer_state = TableState::default();
        dialer_state.select(Some(0));

        Self {
            screen: Screen::Dialer,
            should_quit: false,
            phonebook: hardcoded_phonebook(),
            dialer_state,
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer => render_dialer(frame, self),
            // All other screens are stubs — render a placeholder.
            _ => render_stub(frame),
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer => self.dialer_input(key),
            _ => {
                if key.code == KeyCode::Esc {
                    self.screen = Screen::Dialer;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        // Drive animations, dial phases, baud-rate output — stub for now.
    }

    // ── Dialer input ────────────────────────────────────────────────────────

    fn dialer_input(&mut self, key: KeyEvent) {
        let len = self.phonebook.len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let next = self
                    .dialer_state
                    .selected()
                    .map(|i| if i + 1 >= len { 0 } else { i + 1 })
                    .unwrap_or(0);
                self.dialer_state.select(Some(next));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let prev = self
                    .dialer_state
                    .selected()
                    .map(|i| if i == 0 { len.saturating_sub(1) } else { i - 1 })
                    .unwrap_or(0);
                self.dialer_state.select(Some(prev));
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(idx) = self.dialer_state.selected() {
                    if let Some(bbs) = self.phonebook.get(idx).cloned() {
                        self.screen = Screen::Dialing {
                            bbs,
                            phase: DialPhase::PickingUpLine,
                        };
                    }
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Placeholder render for unimplemented screens
// ---------------------------------------------------------------------------

fn render_stub(frame: &mut Frame) {
    use ratatui::{
        style::{Color, Style},
        text::Line,
        widgets::{Block, Borders, Paragraph},
    };
    let msg = Paragraph::new(Line::from("  Screen not yet implemented — press [Esc] to return."))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" BBS-SIM ")
                .style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(msg, frame.area());
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
