use macroquad::prelude::*;

use crate::bbs::boards::hardcoded_boards;
use crate::bbs::data::{BbsEntry, Board, Message, Thread};
use crate::bbs::session::Session;
use crate::sim::modem::{DialPhase, ModemSim};
use crate::sim::typer::BaudTyper;
use crate::tui::boards::{render_message_boards, render_read_thread, render_thread_list};
use crate::tui::compose::render_compose;
use crate::tui::dialer::render_dialer;
use crate::tui::dialing::render_dialing;
use crate::tui::login::render_login;
use crate::tui::logout::render_logout;
use crate::tui::menus::render_main_menu;
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
    ThreadList { board_id: String },
    ReadThread { board_id: String, thread_id: u32 },
    ComposeMessage { board_id: String, thread_id: Option<u32> },
    Files,
    Logout,
}

// ---------------------------------------------------------------------------
// ComposeField / ComposeState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ComposeField {
    Subject,
    Body,
}

pub struct ComposeState {
    pub board_id: String,
    pub board_name: String,
    pub thread_id: Option<u32>,
    pub reply_subject: String,
    pub subject: String,
    pub body: String,
    pub field: ComposeField,
}

// ---------------------------------------------------------------------------
// LogoutState
// ---------------------------------------------------------------------------

pub struct LogoutState {
    pub bbs_name: String,
    pub typer: BaudTyper,
    pub buffer: TerminalBuffer,
    pub done: bool,
}

impl LogoutState {
    fn new(bbs_name: String, handle: &str, baud: u32) -> Self {
        let mut typer = BaudTyper::new(baud);
        let msg = format!(
            "\r\n\r\nThank you for calling {}!\r\n\
             Come back soon, {}!\r\n\r\n\
             Disconnecting...\r\n\r\n\
             NO CARRIER\r\n",
            bbs_name, handle
        );
        typer.enqueue(&msg);
        Self {
            bbs_name,
            typer,
            buffer: TerminalBuffer::new(80, 100),
            done: false,
        }
    }
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
    pub boards: Vec<Board>,
    pub selected_board_row: usize,
    pub selected_thread_row: usize,
    pub read_scroll: usize,
    pub compose: Option<ComposeState>,
    pub logout: Option<LogoutState>,
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
            boards: hardcoded_boards(),
            selected_board_row: 0,
            selected_thread_row: 0,
            read_scroll: 0,
            compose: None,
            logout: None,
        }
    }

    pub fn render(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer        => render_dialer(self),
            Screen::Dialing       => render_dialing(self),
            Screen::Login         => render_login(self),
            Screen::MainMenu      => render_main_menu(self),
            Screen::MessageBoards => render_message_boards(self),
            Screen::ThreadList { ref board_id } => render_thread_list(self, board_id),
            Screen::ReadThread { ref board_id, thread_id } => {
                render_read_thread(self, board_id, thread_id)
            }
            Screen::ComposeMessage { .. } => render_compose(self),
            Screen::Logout        => render_logout(self),
            Screen::Files         => render_stub(),
        }
    }

    pub fn handle_input(&mut self) {
        let screen = self.screen.clone();
        match screen {
            Screen::Dialer        => self.dialer_input(),
            Screen::Dialing       => self.dialing_input(),
            Screen::Login         => self.login_input(),
            Screen::MainMenu      => self.main_menu_input(),
            Screen::MessageBoards => self.message_boards_input(),
            Screen::ThreadList { board_id } => self.thread_list_input(board_id),
            Screen::ReadThread { board_id, thread_id } => {
                self.read_thread_input(board_id, thread_id)
            }
            Screen::ComposeMessage { .. } => self.compose_input(),
            Screen::Logout        => self.logout_input(),
            Screen::Files => {
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::MainMenu;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        match self.screen.clone() {
            Screen::Dialing => self.tick_dialing(),
            Screen::Login   => self.tick_login(),
            Screen::Logout  => self.tick_logout(),
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

    // ── Input: main menu ─────────────────────────────────────────────────────

    fn main_menu_input(&mut self) {
        while let Some(ch) = get_char_pressed() {
            match ch.to_ascii_lowercase() {
                'm' => {
                    self.selected_board_row = 0;
                    self.screen = Screen::MessageBoards;
                }
                'f' => {
                    self.screen = Screen::Files;
                }
                'g' | 'l' => {
                    self.begin_logout();
                }
                _ => {}
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.begin_logout();
        }
    }

    // ── Input: message boards ────────────────────────────────────────────────

    fn message_boards_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }

        let len = self.boards.len();

        if is_key_pressed(KeyCode::Up) {
            self.selected_board_row =
                if self.selected_board_row == 0 { len.saturating_sub(1) }
                else { self.selected_board_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_board_row =
                if self.selected_board_row + 1 >= len { 0 }
                else { self.selected_board_row + 1 };
        }

        while let Some(ch) = get_char_pressed() {
            match ch {
                'k' => {
                    self.selected_board_row =
                        if self.selected_board_row == 0 { len.saturating_sub(1) }
                        else { self.selected_board_row - 1 };
                }
                'j' => {
                    self.selected_board_row =
                        if self.selected_board_row + 1 >= len { 0 }
                        else { self.selected_board_row + 1 };
                }
                _ => {}
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(board) = self.boards.get(self.selected_board_row) {
                let board_id = board.id.clone();
                self.selected_thread_row = 0;
                self.screen = Screen::ThreadList { board_id };
            }
        }
    }

    // ── Input: thread list ───────────────────────────────────────────────────

    fn thread_list_input(&mut self, board_id: String) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MessageBoards;
            return;
        }

        let len = self.boards.iter()
            .find(|b| b.id == board_id)
            .map(|b| b.threads.len())
            .unwrap_or(0);

        if is_key_pressed(KeyCode::Up) {
            self.selected_thread_row =
                if self.selected_thread_row == 0 { len.saturating_sub(1) }
                else { self.selected_thread_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_thread_row =
                if self.selected_thread_row + 1 >= len { 0 }
                else { self.selected_thread_row + 1 };
        }

        while let Some(ch) = get_char_pressed() {
            match ch {
                'k' => {
                    self.selected_thread_row =
                        if self.selected_thread_row == 0 { len.saturating_sub(1) }
                        else { self.selected_thread_row - 1 };
                }
                'j' => {
                    self.selected_thread_row =
                        if self.selected_thread_row + 1 >= len { 0 }
                        else { self.selected_thread_row + 1 };
                }
                'n' => {
                    if let Some(board) = self.boards.iter().find(|b| b.id == board_id) {
                        let board_name = board.name.clone();
                        self.compose = Some(ComposeState {
                            board_id: board_id.clone(),
                            board_name,
                            thread_id: None,
                            reply_subject: String::new(),
                            subject: String::new(),
                            body: String::new(),
                            field: ComposeField::Subject,
                        });
                        self.screen = Screen::ComposeMessage {
                            board_id,
                            thread_id: None,
                        };
                        return;
                    }
                }
                _ => {}
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(board) = self.boards.iter().find(|b| b.id == board_id) {
                if let Some(thread) = board.threads.get(self.selected_thread_row) {
                    let thread_id = thread.id;
                    self.read_scroll = 0;
                    self.screen = Screen::ReadThread { board_id, thread_id };
                }
            }
        }
    }

    // ── Input: read thread ───────────────────────────────────────────────────

    fn read_thread_input(&mut self, board_id: String, thread_id: u32) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::ThreadList { board_id };
            return;
        }
        if is_key_pressed(KeyCode::Up) {
            self.read_scroll = self.read_scroll.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.read_scroll += 1;
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.read_scroll = self.read_scroll.saturating_sub(10);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.read_scroll += 10;
        }

        while let Some(ch) = get_char_pressed() {
            if ch == 'r' {
                let board_name = self.boards.iter()
                    .find(|b| b.id == board_id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let reply_subject = self.boards.iter()
                    .find(|b| b.id == board_id)
                    .and_then(|b| b.threads.iter().find(|t| t.id == thread_id))
                    .map(|t| t.subject.clone())
                    .unwrap_or_default();
                self.compose = Some(ComposeState {
                    board_id: board_id.clone(),
                    board_name,
                    thread_id: Some(thread_id),
                    reply_subject,
                    subject: String::new(),
                    body: String::new(),
                    field: ComposeField::Body,
                });
                self.screen = Screen::ComposeMessage {
                    board_id,
                    thread_id: Some(thread_id),
                };
                return;
            }
        }
    }

    // ── Input: compose ───────────────────────────────────────────────────────

    fn compose_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            let (board_id, thread_id) = self.compose.as_ref()
                .map(|c| (c.board_id.clone(), c.thread_id))
                .unwrap_or_default();
            self.compose = None;
            self.screen = match thread_id {
                Some(tid) => Screen::ReadThread { board_id, thread_id: tid },
                None      => Screen::ThreadList { board_id },
            };
            return;
        }

        if is_key_pressed(KeyCode::F1) {
            self.do_post();
            return;
        }

        let Some(ref mut compose) = self.compose else { return };

        if compose.thread_id.is_none() && is_key_pressed(KeyCode::Tab) {
            compose.field = match compose.field {
                ComposeField::Subject => ComposeField::Body,
                ComposeField::Body    => ComposeField::Subject,
            };
            return;
        }

        if is_key_pressed(KeyCode::Enter) {
            match compose.field {
                ComposeField::Subject => { compose.field = ComposeField::Body; }
                ComposeField::Body    => { compose.body.push('\n'); }
            }
            return;
        }

        if is_key_pressed(KeyCode::Backspace) {
            match compose.field {
                ComposeField::Subject => { compose.subject.pop(); }
                ComposeField::Body    => { compose.body.pop(); }
            }
            return;
        }

        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii() && !ch.is_control() {
                match compose.field {
                    ComposeField::Subject => {
                        if compose.subject.len() < 60 {
                            compose.subject.push(ch);
                        }
                    }
                    ComposeField::Body => {
                        compose.body.push(ch);
                    }
                }
            }
        }
    }

    fn do_post(&mut self) {
        let Some(ref compose) = self.compose else { return };
        if compose.thread_id.is_none() && compose.subject.trim().is_empty() { return; }
        if compose.body.trim().is_empty() { return; }

        let handle    = self.session.user_handle.clone().unwrap_or_else(|| "Anonymous".into());
        let timestamp = "04/27/93 12:00".to_string();
        let board_id  = compose.board_id.clone();
        let thread_id = compose.thread_id;
        let subject   = compose.subject.clone();
        let body      = compose.body.trim_end().to_string();

        if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
            if let Some(tid) = thread_id {
                if let Some(thread) = board.threads.iter_mut().find(|t| t.id == tid) {
                    let msg_id = thread.posts.len() as u32 + 1;
                    thread.posts.push(Message {
                        id: msg_id,
                        author: handle,
                        subject: format!("Re: {}", thread.subject),
                        body,
                        timestamp,
                    });
                }
                self.compose    = None;
                self.read_scroll = usize::MAX; // clamped in render
                self.screen = Screen::ReadThread { board_id, thread_id: tid };
            } else {
                let new_tid = board.threads.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                board.threads.push(Thread {
                    id: new_tid,
                    subject: subject.clone(),
                    posts: vec![Message {
                        id: 1,
                        author: handle,
                        subject,
                        body,
                        timestamp,
                    }],
                });
                let last = board.threads.len() - 1;
                self.compose             = None;
                self.selected_thread_row = last;
                self.screen = Screen::ThreadList { board_id };
            }
        }
    }

    // ── Input / tick: logout ─────────────────────────────────────────────────

    fn logout_input(&mut self) {
        let done = self.logout.as_ref().map(|s| s.done).unwrap_or(false);
        if is_key_pressed(KeyCode::Escape)
            || (done && (is_key_pressed(KeyCode::Enter) || get_char_pressed().is_some()))
        {
            self.finish_logout();
        }
    }

    fn tick_logout(&mut self) {
        let green = Color::new(0.0, 0.85, 0.0, 1.0);
        if let Some(ref mut state) = self.logout {
            for ch in state.typer.tick() {
                state.buffer.push_char(ch, CellStyle::fg(green));
            }
            if state.typer.is_empty() {
                state.done = true;
            }
        }
    }

    fn begin_logout(&mut self) {
        let bbs_name = self.session.bbs_name.clone().unwrap_or_default();
        let handle   = self.session.user_handle.clone().unwrap_or_default();
        let baud     = self.phonebook.iter()
            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
            .map(|e| e.baud)
            .unwrap_or(2400);
        self.logout = Some(LogoutState::new(bbs_name, &handle, baud));
        self.screen = Screen::Logout;
    }

    fn finish_logout(&mut self) {
        if let Some(ref state) = self.logout {
            let bbs_name = state.bbs_name.clone();
            if let Some(entry) = self.phonebook.iter_mut().find(|e| e.name == bbs_name) {
                entry.last_called = Some("04/27/93".into());
            }
        }
        self.session.logout();
        self.logout = None;
        self.screen = Screen::Dialer;
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
