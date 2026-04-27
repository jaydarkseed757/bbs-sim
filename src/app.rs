use macroquad::prelude::*;

use crate::bbs::data::BbsEntry;
use crate::bbs::session::Session;
use crate::sim::modem::{DialPhase, ModemSim};
use crate::sim::typer::BaudTyper;
use crate::tui::dialer::render_dialer;
use crate::tui::dialing::render_dialing;
use crate::tui::login::render_login;
use crate::tui::terminal::{CellStyle, TerminalBuffer};

// Palette constants reused across tick methods.
const GREEN_BBS: Color    = Color::new(0.0,  0.85, 0.0,  1.0); // modem / dialing output
const WHITE_BBS: Color    = Color::new(0.85, 0.85, 0.85, 1.0); // login system text
const INPUT_GREEN: Color  = Color::new(0.0,  0.90, 0.0,  1.0); // user-typed chars

// ---------------------------------------------------------------------------
// Screen state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Screen {
    Dialer,
    Dialing,
    Login,
    MainMenu,
    MessageBoards,
    ReadThread { board_id: String, thread_id: u32 },
    ComposeMessage,
    Files,
    Logout,
}

// ---------------------------------------------------------------------------
// Login step sub-state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum LoginStep {
    Banner,        // welcome text typing through baud typer
    HandlePrompt,  // "Enter your handle: " typing out
    Handle,        // waiting for the user to type a handle + Enter
    PassPrompt,    // "Enter your password: " typing out
    Password,      // waiting for the user to type a password + Enter
    WelcomeText,   // "Access granted! Welcome, {handle}!" typing out
    Done,          // ready to transition to MainMenu
}

// ---------------------------------------------------------------------------
// DialingState
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
        Self { bbs, modem, typer, buffer: TerminalBuffer::new(80, 500), awaiting_keypress: false }
    }
}

// ---------------------------------------------------------------------------
// LoginState
// ---------------------------------------------------------------------------

pub struct LoginState {
    pub bbs: BbsEntry,
    pub typer: BaudTyper,
    pub buffer: TerminalBuffer,
    pub input: String,
    pub step: LoginStep,
    pub user_handle: String,
    /// Column at which user input begins on the current prompt line.
    pub input_start_col: usize,
    pub cursor_blink: bool,
    blink_ticks: u32,
}

impl LoginState {
    fn new(bbs: BbsEntry) -> Self {
        let mut typer = BaudTyper::new(bbs.baud);

        let boards = bbs.boards.join("  ");
        let sep = "-".repeat(42);
        let banner = format!(
            "\r\n{sep}\r\n  {name}\r\n{sep}\r\n\r\n\
             Sysop:    {sysop}\r\n\
             Location: {location}\r\n\
             Boards:   {boards}\r\n\r\n",
            sep = sep,
            name = bbs.name,
            sysop = bbs.sysop,
            location = bbs.location,
            boards = boards,
        );
        typer.enqueue(&banner);

        Self {
            bbs,
            typer,
            buffer: TerminalBuffer::new(80, 500),
            input: String::new(),
            step: LoginStep::Banner,
            user_handle: String::new(),
            input_start_col: 0,
            cursor_blink: true,
            blink_ticks: 0,
        }
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
    pub login: Option<LoginState>,
    pub session: Session,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dialer,
            should_quit: false,
            phonebook: hardcoded_phonebook(),
            selected_row: 0,
            dial: None,
            login: None,
            session: Session::new(),
        }
    }

    pub fn render(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer  => render_dialer(self),
            Screen::Dialing => render_dialing(self),
            Screen::Login   => render_login(self),
            _               => render_stub(),
        }
    }

    pub fn handle_input(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer  => self.dialer_input(),
            Screen::Dialing => self.dialing_input(),
            Screen::Login   => self.login_input(),
            _ => {
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Dialer;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        match self.screen.clone() {
            Screen::Dialing => self.tick_dialing(),
            Screen::Login   => self.tick_login(),
            _               => {}
        }
    }

    // ── Tick: dialing ────────────────────────────────────────────────────────

    fn tick_dialing(&mut self) {
        let Some(ref mut d) = self.dial else { return };

        if let Some(text) = d.modem.tick() {
            d.typer.enqueue(&text);
        }
        for ch in d.typer.tick() {
            d.buffer.push_char(ch, CellStyle::fg(GREEN_BBS));
        }
        if d.modem.phase == DialPhase::Connected && d.typer.is_empty() && !d.awaiting_keypress {
            d.buffer.push_str("\r\n\r\nPress [Enter] to log in...\r\n", CellStyle::fg(YELLOW));
            d.awaiting_keypress = true;
        }
    }

    // ── Tick: login ──────────────────────────────────────────────────────────

    fn tick_login(&mut self) {
        let mut transition: Option<(String, String)> = None;

        if let Some(ref mut d) = self.login {
            // Drain the baud typer into the buffer.
            for ch in d.typer.tick() {
                d.buffer.push_char(ch, CellStyle::fg(WHITE_BBS));
            }

            // Cursor blink: toggle every 10 ticks (~0.5 s at 20 Hz).
            d.blink_ticks += 1;
            if d.blink_ticks >= 10 {
                d.cursor_blink = !d.cursor_blink;
                d.blink_ticks = 0;
            }

            // State transitions — only advance when the typer queue is fully drained.
            match d.step.clone() {
                LoginStep::Banner if d.typer.is_empty() => {
                    d.typer.enqueue("Enter your handle: ");
                    d.step = LoginStep::HandlePrompt;
                }
                LoginStep::HandlePrompt if d.typer.is_empty() => {
                    d.input_start_col = d.buffer.cursor_col;
                    d.step = LoginStep::Handle;
                }
                LoginStep::PassPrompt if d.typer.is_empty() => {
                    d.input_start_col = d.buffer.cursor_col;
                    d.step = LoginStep::Password;
                }
                LoginStep::WelcomeText if d.typer.is_empty() => {
                    d.step = LoginStep::Done;
                }
                LoginStep::Done => {
                    transition = Some((d.user_handle.clone(), d.bbs.name.clone()));
                }
                _ => {}
            }
        }

        if let Some((handle, bbs_name)) = transition {
            self.session.login(handle, bbs_name);
            self.login = None;
            self.screen = Screen::MainMenu;
        }
    }

    // ── Input: dialer ────────────────────────────────────────────────────────

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

    // ── Input: dialing ───────────────────────────────────────────────────────

    fn dialing_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Dialer;
            self.dial = None;
            return;
        }

        let awaiting = self.dial.as_ref().map(|d| d.awaiting_keypress).unwrap_or(false);
        if awaiting && is_key_pressed(KeyCode::Enter) {
            if let Some(d) = self.dial.take() {
                self.login = Some(LoginState::new(d.bbs));
                self.screen = Screen::Login;
            }
        }
    }

    // ── Input: login ─────────────────────────────────────────────────────────

    fn login_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Dialer;
            self.login = None;
            return;
        }

        let Some(ref mut d) = self.login else { return };

        // Only accept keystrokes when we're actually waiting for input.
        if !matches!(d.step, LoginStep::Handle | LoginStep::Password) {
            return;
        }

        let step = d.step.clone();

        // Backspace — don't erase past the start of the current input field.
        if is_key_pressed(KeyCode::Backspace) {
            if !d.input.is_empty() {
                d.input.pop();
                d.buffer.backspace();
            }
            return;
        }

        // Enter — commit the current field.
        if is_key_pressed(KeyCode::Enter) {
            match step {
                LoginStep::Handle => {
                    if d.input.is_empty() {
                        return;
                    }
                    d.user_handle = d.input.clone();
                    d.input.clear();
                    d.buffer.push_str("\r\n", CellStyle::fg(WHITE_BBS));
                    d.typer.enqueue("Enter your password: ");
                    d.step = LoginStep::PassPrompt;
                }
                LoginStep::Password => {
                    d.input.clear();
                    let msg = format!(
                        "\r\n\r\nAccess granted!  Welcome, {}!\r\n\r\n",
                        d.user_handle
                    );
                    d.buffer.push_str("\r\n", CellStyle::fg(WHITE_BBS));
                    d.typer.enqueue(&msg);
                    d.step = LoginStep::WelcomeText;
                }
                _ => {}
            }
            return;
        }

        // Printable chars — echo handle; mask password as '*'.
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii() && !ch.is_control() && d.input.len() < 30 {
                d.input.push(ch);
                let echo = if step == LoginStep::Password { '*' } else { ch };
                d.buffer.push_char(echo, CellStyle::fg(INPUT_GREEN));
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
