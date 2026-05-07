use macroquad::prelude::*;

use crate::bbs::boards::{hardcoded_boards, load_boards};
use crate::bbs::data::{BbsEntry, Board, FileSection, MailMessage, Message, Oneliner, Poll, Thread, TopLists};
use crate::bbs::files::load_files;
use crate::bbs::mail::load_mail;
use crate::bbs::oneliners::load_oneliners;
use crate::bbs::session::Session;
use crate::bbs::top10::load_top10;
use crate::bbs::voting::load_polls;
use crate::sim::modem::{DialPhase, ModemSim};
use crate::sim::typer::BaudTyper;
use crate::tui::boards::{render_message_boards, render_read_thread, render_thread_list};
use crate::tui::compose::render_compose;
use crate::tui::files::{render_file_list, render_view_file};
use crate::tui::graffiti::render_graffiti_wall;
use crate::tui::mail::{render_compose_mail, render_inbox, render_read_mail};
use crate::tui::top10::render_top10;
use crate::tui::dialer::render_dialer;
use crate::tui::dialing::render_dialing;
use crate::tui::login::render_login;
use crate::tui::logout::render_logout;
use crate::tui::menus::render_main_menu;
use crate::tui::download::render_download;
use crate::tui::theme::Theme;
use crate::tui::voting::render_voting;
use crate::tui::sysop::render_sysop_chat;
use crate::tui::terminal::{CellStyle, TerminalBuffer};

// Palette constants reused across tick methods.
const GREEN_BBS: Color    = Color::new(0.0,  0.85, 0.0,  1.0); // modem / dialing output
const WHITE_BBS: Color    = Color::new(0.85, 0.85, 0.85, 1.0); // login system text
const INPUT_GREEN: Color  = Color::new(0.0,  0.90, 0.0,  1.0); // user-typed chars

// Voting scroll helpers — mirror tui/voting.rs POLL_H_VOTED / POLL_H_UNVOTED.
const VOTE_H_VOTED:   f32 = 18.0 * 1.35 * 3.8;
const VOTE_H_UNVOTED: f32 = 18.0 * 1.35 * 2.8;
const VOTE_VIEWPORT:  f32 = 450.0; // conservative body-height budget (px)

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
    ComposeMessage,
    Mail,
    ReadMail { id: u32 },
    ComposeMail,
    Files,
    ViewFile { id: u32 },
    Downloading,
    GraffitiWall,
    TopTen,
    Voting,
    SysopChat,
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
// ComposeMailField / ComposeMailState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ComposeMailField {
    To,
    Subject,
    Body,
}

pub struct ComposeMailState {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub field: ComposeMailField,
    pub reply_to_id: Option<u32>,
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
// SysopChatState
// ---------------------------------------------------------------------------

pub struct SysopChatState {
    pub bbs_name: String,
    pub typer: BaudTyper,
    pub buffer: TerminalBuffer,
    pub done: bool,
}

impl SysopChatState {
    fn new(bbs_name: String, sysop: &str, baud: u32) -> Self {
        let mut typer = BaudTyper::new(baud);
        let msg = format!(
            "\r\nPaging sysop: {} ...\r\n\
             \r\n\
             *BEEP* *BEEP* *BEEP*\r\n\
             \r\n\
             No response.\r\n\
             \r\n\
             The sysop is not available right now.\r\n\
             Please leave a message in the mail system.\r\n",
            sysop
        );
        typer.enqueue(&msg);
        Self { bbs_name, typer, buffer: TerminalBuffer::new(80, 100), done: false }
    }
}

// ---------------------------------------------------------------------------
// DownloadState
// ---------------------------------------------------------------------------

fn parse_size(s: &str) -> u64 {
    let s = s.trim();
    if let Some(n) = s.strip_suffix("MB").or_else(|| s.strip_suffix('M')) {
        (n.trim().parse::<f64>().unwrap_or(0.0) * 1_048_576.0) as u64
    } else if let Some(n) = s.strip_suffix("KB").or_else(|| s.strip_suffix('K')) {
        (n.trim().parse::<f64>().unwrap_or(0.0) * 1_024.0) as u64
    } else {
        s.parse::<u64>().unwrap_or(10_240)
    }
}

pub struct DownloadState {
    pub file_name: String,
    pub total_bytes: u64,
    pub transferred: u64,
    pub baud: u32,
    pub elapsed_ticks: u64,
    pub packet_count: u64,
    pub bytes_per_tick: u64,
    pub done: bool,
}

impl DownloadState {
    fn new(file_name: String, size_str: &str, baud: u32) -> Self {
        let total_bytes = parse_size(size_str).max(1);
        // Target ~15 seconds at 20 ticks/sec = 300 ticks
        let bytes_per_tick = (total_bytes / 300).max(1);
        Self {
            file_name,
            total_bytes,
            transferred: 0,
            baud,
            elapsed_ticks: 0,
            packet_count: 0,
            bytes_per_tick,
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
    pub no_answer: bool,
}

impl DialingState {
    fn new(bbs: BbsEntry) -> Self {
        let modem = ModemSim::new(&bbs.number);
        let typer = BaudTyper::new(bbs.baud);
        Self { bbs, modem, typer, buffer: TerminalBuffer::new(80, 500), awaiting_keypress: false, no_answer: false }
    }

    fn new_manual(number: String) -> Self {
        let modem = ModemSim::new_no_answer(&number);
        let typer = BaudTyper::new(2400);
        let bbs = BbsEntry {
            name: String::new(),
            number,
            sysop: String::new(),
            location: String::new(),
            baud: 2400,
            boards: vec![],
            last_called: None,
            slug: String::new(),
            total_callers: 0,
            call_count: 0,
        };
        Self { bbs, modem, typer, buffer: TerminalBuffer::new(80, 500), awaiting_keypress: false, no_answer: true }
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
    pub sysop_chat: Option<SysopChatState>,
    pub mail: Vec<MailMessage>,
    pub selected_mail_row: usize,
    pub mail_read_scroll: usize,
    pub compose_mail: Option<ComposeMailState>,
    pub files: Vec<FileSection>,
    pub selected_file_row: usize,
    pub file_view_scroll: usize,
    pub download: Option<DownloadState>,
    pub manual_dial_input: Option<String>,
    pub oneliners: Vec<Oneliner>,
    pub graffiti_scroll: usize,
    pub graffiti_input: Option<String>,
    pub theme: Theme,
    pub top_lists: TopLists,
    pub top_list_tab: usize,
    pub polls: Vec<Poll>,
    pub selected_poll_row: usize,
    pub voting_scroll: usize,
    pub voted_polls: std::collections::HashSet<u32>,
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
            sysop_chat: None,
            mail: vec![],
            selected_mail_row: 0,
            mail_read_scroll: 0,
            compose_mail: None,
            files: vec![],
            selected_file_row: 0,
            file_view_scroll: 0,
            download: None,
            manual_dial_input: None,
            oneliners: vec![],
            graffiti_scroll: 0,
            graffiti_input: None,
            theme: Theme::for_slug(""),
            top_lists: TopLists::default(),
            top_list_tab: 0,
            polls: vec![],
            selected_poll_row: 0,
            voting_scroll: 0,
            voted_polls: std::collections::HashSet::new(),
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
            Screen::Mail          => render_inbox(self),
            Screen::ReadMail { id } => render_read_mail(self, id),
            Screen::ComposeMail   => render_compose_mail(self),
            Screen::Files           => render_file_list(self),
            Screen::ViewFile { id } => render_view_file(self, id),
            Screen::Downloading { .. } => render_download(self),
            Screen::GraffitiWall    => render_graffiti_wall(self),
            Screen::TopTen          => render_top10(self),
            Screen::Voting          => render_voting(self),
            Screen::SysopChat       => render_sysop_chat(self),
            Screen::Logout        => render_logout(self),
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
            Screen::Mail          => self.inbox_input(),
            Screen::ReadMail { id } => self.read_mail_input(id),
            Screen::ComposeMail   => self.compose_mail_input(),
            Screen::Files           => self.files_input(),
            Screen::ViewFile { id } => self.view_file_input(id),
            Screen::Downloading { .. } => self.download_input(),
            Screen::GraffitiWall    => self.graffiti_wall_input(),
            Screen::TopTen          => self.top10_input(),
            Screen::Voting          => self.voting_input(),
            Screen::SysopChat       => self.sysop_chat_input(),
            Screen::Logout        => self.logout_input(),
        }
    }

    pub fn tick(&mut self) {
        match self.screen.clone() {
            Screen::Dialing        => self.tick_dialing(),
            Screen::Login          => self.tick_login(),
            Screen::Logout         => self.tick_logout(),
            Screen::SysopChat      => self.tick_sysop_chat(),
            Screen::Downloading { .. } => self.tick_download(),
            _                      => {}
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
        if d.no_answer && d.modem.phase == DialPhase::NoAnswer && d.typer.is_empty() && !d.awaiting_keypress {
            d.buffer.push_str("\r\nPress any key to return...\r\n", CellStyle::fg(YELLOW));
            d.awaiting_keypress = true;
        }
    }

    // ── Tick: login ──────────────────────────────────────────────────────────

    fn tick_login(&mut self) {
        let mut transition: Option<(String, String, String)> = None;

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
                    transition = Some((
                        d.user_handle.clone(),
                        d.bbs.name.clone(),
                        d.bbs.slug.clone(),
                    ));
                }
                _ => {}
            }
        }

        if let Some((handle, bbs_name, slug)) = transition {
            self.session.login(handle.clone(), bbs_name.clone());
            self.theme      = Theme::for_slug(&slug);
            self.boards     = load_boards(&slug);
            self.mail       = load_mail(&slug, &handle);
            self.files      = load_files(&slug);
            self.oneliners  = load_oneliners(&slug);
            self.graffiti_scroll = self.oneliners.len().saturating_sub(1);
            self.top_lists  = load_top10(&slug);
            self.top_list_tab = 0;
            self.polls = load_polls(&slug);
            self.selected_poll_row = 0;
            self.voting_scroll = 0;
            self.voted_polls.clear();
            self.selected_board_row  = 0;
            self.selected_thread_row = 0;
            self.selected_mail_row   = 0;
            self.selected_file_row   = 0;
            if let Some(entry) = self.phonebook.iter_mut().find(|e| e.name == bbs_name) {
                entry.call_count  += 1;
                entry.last_called  = Some("04/28/93".into());
            }
            self.login = None;
            self.screen = Screen::MainMenu;
        }
    }

    // ── Input: dialer ────────────────────────────────────────────────────────

    fn dialer_input(&mut self) {
        // ── Manual dial overlay input ─────────────────────────────────────────
        if self.manual_dial_input.is_some() {
            if is_key_pressed(KeyCode::Escape) {
                self.manual_dial_input = None;
                return;
            }
            if is_key_pressed(KeyCode::Backspace) {
                if let Some(ref mut s) = self.manual_dial_input {
                    s.pop();
                }
                return;
            }
            if is_key_pressed(KeyCode::Enter) {
                let number = self.manual_dial_input.take().unwrap_or_default();
                if !number.trim().is_empty() {
                    self.dial = Some(DialingState::new_manual(number));
                    self.screen = Screen::Dialing;
                }
                return;
            }
            while let Some(ch) = get_char_pressed() {
                if let Some(ref mut s) = self.manual_dial_input {
                    if s.len() < 20 && (ch.is_ascii_digit() || matches!(ch, '-' | '+' | ' ' | '(' | ')')) {
                        s.push(ch);
                    }
                }
            }
            return;
        }

        // ── Normal phonebook input ────────────────────────────────────────────
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
                'm' | 'M' => {
                    self.manual_dial_input = Some(String::new());
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

        let awaiting   = self.dial.as_ref().map(|d| d.awaiting_keypress).unwrap_or(false);
        let no_answer  = self.dial.as_ref().map(|d| d.no_answer).unwrap_or(false);

        if awaiting && no_answer {
            if is_key_pressed(KeyCode::Enter) || get_char_pressed().is_some() {
                self.dial = None;
                self.screen = Screen::Dialer;
            }
        } else if awaiting && is_key_pressed(KeyCode::Enter) {
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
                    let this_call = d.bbs.call_count + 1;
                    let times_str = if this_call == 1 {
                        "1 time".to_string()
                    } else {
                        format!("{} times", this_call)
                    };
                    let last_on = match d.bbs.last_called.as_deref() {
                        Some(date) => format!("Last on: {}.", date),
                        None       => "First call!".to_string(),
                    };
                    let msg = format!(
                        "\r\n\r\nAccess granted!  Welcome, {}!\r\n\r\n\
                         You are caller #{}.  You have called {}.  {}\r\n\r\n",
                        d.user_handle,
                        d.bbs.total_callers + this_call,
                        times_str,
                        last_on,
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
                'b' => {
                    self.selected_board_row = 0;
                    self.screen = Screen::MessageBoards;
                }
                'm' => {
                    self.selected_mail_row = 0;
                    self.screen = Screen::Mail;
                }
                'f' => {
                    self.selected_file_row = 0;
                    self.screen = Screen::Files;
                }
                'o' => {
                    self.graffiti_input = None;
                    self.screen = Screen::GraffitiWall;
                }
                't' => {
                    self.top_list_tab = 0;
                    self.screen = Screen::TopTen;
                }
                'v' => {
                    self.selected_poll_row = 0;
                    self.voting_scroll = 0;
                    self.screen = Screen::Voting;
                }
                'c' => {
                    self.begin_sysop_chat();
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
                        self.screen = Screen::ComposeMessage;
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
                self.screen = Screen::ComposeMessage;
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

    // ── Input: files ─────────────────────────────────────────────────────────

    fn files_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }

        let flat_count: usize = self.files.iter().map(|s| s.files.len()).sum();

        if is_key_pressed(KeyCode::Up) {
            self.selected_file_row =
                if self.selected_file_row == 0 { flat_count.saturating_sub(1) }
                else { self.selected_file_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_file_row =
                if flat_count == 0 || self.selected_file_row + 1 >= flat_count { 0 }
                else { self.selected_file_row + 1 };
        }

        while let Some(ch) = get_char_pressed() {
            match ch.to_ascii_lowercase() {
                'k' => {
                    self.selected_file_row =
                        if self.selected_file_row == 0 { flat_count.saturating_sub(1) }
                        else { self.selected_file_row - 1 };
                }
                'j' => {
                    self.selected_file_row =
                        if flat_count == 0 || self.selected_file_row + 1 >= flat_count { 0 }
                        else { self.selected_file_row + 1 };
                }
                'v' => {
                    let file = self.files.iter()
                        .flat_map(|s| s.files.iter())
                        .nth(self.selected_file_row)
                        .cloned();
                    if let Some(f) = file {
                        if f.kind == "text" {
                            self.file_view_scroll = 0;
                            self.screen = Screen::ViewFile { id: f.id };
                            return;
                        }
                    }
                }
                'd' => {
                    let file = self.files.iter()
                        .flat_map(|s| s.files.iter())
                        .nth(self.selected_file_row)
                        .cloned();
                    if let Some(f) = file {
                        let id = f.id;
                        self.begin_download(id);
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    fn view_file_input(&mut self, _id: u32) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Files;
            return;
        }
        if is_key_pressed(KeyCode::Up) {
            self.file_view_scroll = self.file_view_scroll.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.file_view_scroll += 1;
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.file_view_scroll = self.file_view_scroll.saturating_sub(10);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.file_view_scroll += 10;
        }
    }

    // ── Input: inbox ─────────────────────────────────────────────────────────

    fn inbox_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }

        let len = self.mail.len();
        if is_key_pressed(KeyCode::Up) {
            self.selected_mail_row =
                if self.selected_mail_row == 0 { len.saturating_sub(1) }
                else { self.selected_mail_row - 1 };
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_mail_row =
                if self.selected_mail_row + 1 >= len { 0 }
                else { self.selected_mail_row + 1 };
        }

        while let Some(ch) = get_char_pressed() {
            match ch {
                'k' => {
                    self.selected_mail_row =
                        if self.selected_mail_row == 0 { len.saturating_sub(1) }
                        else { self.selected_mail_row - 1 };
                }
                'j' => {
                    self.selected_mail_row =
                        if self.selected_mail_row + 1 >= len { 0 }
                        else { self.selected_mail_row + 1 };
                }
                'c' => {
                    self.compose_mail = Some(ComposeMailState {
                        to: String::new(),
                        subject: String::new(),
                        body: String::new(),
                        field: ComposeMailField::To,
                        reply_to_id: None,
                    });
                    self.screen = Screen::ComposeMail;
                    return;
                }
                _ => {}
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(msg) = self.mail.get_mut(self.selected_mail_row) {
                let id = msg.id;
                msg.read = true;
                self.mail_read_scroll = 0;
                self.screen = Screen::ReadMail { id };
            }
        }
    }

    // ── Input: read mail ──────────────────────────────────────────────────────

    fn read_mail_input(&mut self, id: u32) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Mail;
            return;
        }
        if is_key_pressed(KeyCode::Up) {
            self.mail_read_scroll = self.mail_read_scroll.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.mail_read_scroll += 1;
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.mail_read_scroll = self.mail_read_scroll.saturating_sub(10);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.mail_read_scroll += 10;
        }

        while let Some(ch) = get_char_pressed() {
            if ch == 'r' {
                let (from, subject) = self.mail.iter()
                    .find(|m| m.id == id)
                    .map(|m| (m.from.clone(), format!("Re: {}", m.subject)))
                    .unwrap_or_default();
                self.compose_mail = Some(ComposeMailState {
                    to: from,
                    subject,
                    body: String::new(),
                    field: ComposeMailField::Body,
                    reply_to_id: Some(id),
                });
                self.screen = Screen::ComposeMail;
                return;
            }
        }
    }

    // ── Input: compose mail ───────────────────────────────────────────────────

    fn compose_mail_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.compose_mail = None;
            self.screen = Screen::Mail;
            return;
        }

        if is_key_pressed(KeyCode::F1) {
            self.do_send_mail();
            return;
        }

        let Some(ref mut c) = self.compose_mail else { return };

        if is_key_pressed(KeyCode::Tab) {
            c.field = match c.field {
                ComposeMailField::To      => ComposeMailField::Subject,
                ComposeMailField::Subject => ComposeMailField::Body,
                ComposeMailField::Body    => ComposeMailField::To,
            };
            return;
        }

        if is_key_pressed(KeyCode::Enter) {
            match c.field {
                ComposeMailField::To      => { c.field = ComposeMailField::Subject; }
                ComposeMailField::Subject => { c.field = ComposeMailField::Body; }
                ComposeMailField::Body    => { c.body.push('\n'); }
            }
            return;
        }

        if is_key_pressed(KeyCode::Backspace) {
            match c.field {
                ComposeMailField::To      => { c.to.pop(); }
                ComposeMailField::Subject => { c.subject.pop(); }
                ComposeMailField::Body    => { c.body.pop(); }
            }
            return;
        }

        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii() && !ch.is_control() {
                match c.field {
                    ComposeMailField::To      => { if c.to.len() < 30 { c.to.push(ch); } }
                    ComposeMailField::Subject => { if c.subject.len() < 60 { c.subject.push(ch); } }
                    ComposeMailField::Body    => { c.body.push(ch); }
                }
            }
        }
    }

    fn do_send_mail(&mut self) {
        let Some(ref c) = self.compose_mail else { return };
        if c.to.trim().is_empty() || c.body.trim().is_empty() { return; }

        let next_id = self.mail.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let from    = self.session.user_handle.clone().unwrap_or_else(|| "Anonymous".into());
        let sent    = MailMessage {
            id: next_id,
            from: from.clone(),
            to: c.to.clone(),
            subject: if c.subject.is_empty() { "(no subject)".into() } else { c.subject.clone() },
            body: format!("{}\n\n-- {}", c.body.trim_end(), from),
            timestamp: "04/27/93 12:00".into(),
            read: true,
        };
        self.mail.push(sent);
        self.compose_mail = None;
        self.screen = Screen::Mail;
    }

    // ── Input: graffiti wall ─────────────────────────────────────────────────

    fn graffiti_wall_input(&mut self) {
        let total = self.oneliners.len();

        // Input mode
        if self.graffiti_input.is_some() {
            if is_key_pressed(KeyCode::Escape) {
                self.graffiti_input = None;
                return;
            }
            if is_key_pressed(KeyCode::Enter) {
                let text = self.graffiti_input.take().unwrap_or_default();
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    let handle = self.session.user_handle.clone().unwrap_or_else(|| "Anonymous".into());
                    self.oneliners.push(Oneliner { handle, message: trimmed, date: "04/28/93".into() });
                    self.graffiti_scroll = self.oneliners.len().saturating_sub(1);
                }
                return;
            }
            if is_key_pressed(KeyCode::Backspace) {
                if let Some(ref mut s) = self.graffiti_input { s.pop(); }
                return;
            }
            while let Some(ch) = get_char_pressed() {
                if ch.is_ascii() && !ch.is_control() {
                    if let Some(ref mut s) = self.graffiti_input {
                        if s.len() < 60 { s.push(ch); }
                    }
                }
            }
            return;
        }

        // Normal navigation
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }
        if is_key_pressed(KeyCode::Up) {
            self.graffiti_scroll = self.graffiti_scroll.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) {
            self.graffiti_scroll = (self.graffiti_scroll + 1).min(total.saturating_sub(1));
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.graffiti_scroll = self.graffiti_scroll.saturating_sub(10);
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.graffiti_scroll = (self.graffiti_scroll + 10).min(total.saturating_sub(1));
        }
        while let Some(ch) = get_char_pressed() {
            match ch.to_ascii_lowercase() {
                'k' => { self.graffiti_scroll = self.graffiti_scroll.saturating_sub(1); }
                'j' => { self.graffiti_scroll = (self.graffiti_scroll + 1).min(total.saturating_sub(1)); }
                'a' => { self.graffiti_input = Some(String::new()); }
                _ => {}
            }
        }
    }

    // ── Input: top 10 lists ──────────────────────────────────────────────────

    fn top10_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.top_list_tab = (self.top_list_tab + 1) % 3;
        }
        while let Some(ch) = get_char_pressed() {
            match ch {
                '1' => { self.top_list_tab = 0; }
                '2' => { self.top_list_tab = 1; }
                '3' => { self.top_list_tab = 2; }
                _ => {}
            }
        }
    }

    // ── Input: voting booth ──────────────────────────────────────────────────

    fn voting_input(&mut self) {
        let len = self.polls.len();

        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::MainMenu;
            return;
        }
        if is_key_pressed(KeyCode::Up) {
            if self.selected_poll_row > 0 {
                self.selected_poll_row -= 1;
                self.scroll_voting_to_selected();
            }
        }
        if is_key_pressed(KeyCode::Down) {
            if self.selected_poll_row + 1 < len {
                self.selected_poll_row += 1;
                self.scroll_voting_to_selected();
            }
        }

        while let Some(ch) = get_char_pressed() {
            match ch.to_ascii_lowercase() {
                'k' if self.selected_poll_row > 0 => {
                    self.selected_poll_row -= 1;
                    self.scroll_voting_to_selected();
                }
                'j' if self.selected_poll_row + 1 < len => {
                    self.selected_poll_row += 1;
                    self.scroll_voting_to_selected();
                }
                'y' | 'n' => {
                    if let Some(poll) = self.polls.get_mut(self.selected_poll_row) {
                        if !self.voted_polls.contains(&poll.id) {
                            let id = poll.id;
                            if ch == 'y' { poll.yes_votes += 1; } else { poll.no_votes += 1; }
                            self.voted_polls.insert(id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn scroll_voting_to_selected(&mut self) {
        if self.selected_poll_row < self.voting_scroll {
            self.voting_scroll = self.selected_poll_row;
            return;
        }
        // Sum pixel heights of items between the scroll offset and the selection.
        // Voted items are taller than unvoted, so a count-based threshold is inaccurate.
        let pixel_offset: f32 = self.polls[self.voting_scroll..self.selected_poll_row]
            .iter()
            .map(|p| if self.voted_polls.contains(&p.id) { VOTE_H_VOTED } else { VOTE_H_UNVOTED })
            .sum();
        if pixel_offset >= VOTE_VIEWPORT {
            self.voting_scroll = self.selected_poll_row.saturating_sub(1);
        }
    }

    // ── Input / tick: sysop chat ─────────────────────────────────────────────

    fn begin_sysop_chat(&mut self) {
        let bbs_name = self.session.bbs_name.clone().unwrap_or_default();
        let sysop = self.phonebook.iter()
            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
            .map(|e| e.sysop.clone())
            .unwrap_or_else(|| "Sysop".into());
        let baud = self.phonebook.iter()
            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
            .map(|e| e.baud)
            .unwrap_or(2400);
        self.sysop_chat = Some(SysopChatState::new(bbs_name, &sysop, baud));
        self.screen = Screen::SysopChat;
    }

    fn tick_sysop_chat(&mut self) {
        let green = Color::new(0.0, 0.85, 0.0, 1.0);
        if let Some(ref mut state) = self.sysop_chat {
            for ch in state.typer.tick() {
                state.buffer.push_char(ch, CellStyle::fg(green));
            }
            if state.typer.is_empty() {
                state.done = true;
            }
        }
    }

    fn sysop_chat_input(&mut self) {
        let done = self.sysop_chat.as_ref().map(|s| s.done).unwrap_or(false);
        if is_key_pressed(KeyCode::Escape)
            || (done && (is_key_pressed(KeyCode::Enter) || get_char_pressed().is_some()))
        {
            self.sysop_chat = None;
            self.screen = Screen::MainMenu;
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

    // ── Download ─────────────────────────────────────────────────────────────

    fn begin_download(&mut self, file_id: u32) {
        let baud = self.phonebook.iter()
            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
            .map(|e| e.baud)
            .unwrap_or(2400);
        let file = self.files.iter()
            .flat_map(|s| s.files.iter())
            .find(|f| f.id == file_id)
            .cloned();
        if let Some(f) = file {
            self.download = Some(DownloadState::new(f.name, &f.size, baud));
            self.screen = Screen::Downloading;
        }
    }

    fn tick_download(&mut self) {
        let Some(ref mut dl) = self.download else { return };
        if dl.done { return; }
        dl.elapsed_ticks += 1;
        dl.transferred = (dl.transferred + dl.bytes_per_tick).min(dl.total_bytes);
        dl.packet_count = dl.transferred / 128;
        if dl.transferred >= dl.total_bytes {
            dl.done = true;
        }
    }

    fn download_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.download = None;
            self.screen = Screen::Files;
            return;
        }
        let done = self.download.as_ref().map(|d| d.done).unwrap_or(false);
        if done && (is_key_pressed(KeyCode::Enter) || get_char_pressed().is_some()) {
            self.download = None;
            self.screen = Screen::Files;
        }
    }
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
            slug: "rusty_nail".into(),
            total_callers: 2_341,
            call_count: 14,
        },
        BbsEntry {
            name: "WARP FACTOR 9".into(),
            number: "555-1701".into(),
            sysop: "KirkFan".into(),
            location: "Seattle, WA".into(),
            baud: 9600,
            boards: vec!["SciFi".into(), "Gaming".into(), "Anime".into()],
            last_called: Some("04/15/93".into()),
            slug: "warp_factor_9".into(),
            total_callers: 1_108,
            call_count: 7,
        },
        BbsEntry {
            name: "The Digital Dungeon".into(),
            number: "555-4096".into(),
            sysop: "DungeonMstr".into(),
            location: "Austin, TX".into(),
            baud: 2400,
            boards: vec!["RPG".into(), "D&D".into(), "General".into()],
            last_called: None,
            slug: "digital_dungeon".into(),
            total_callers: 894,
            call_count: 0,
        },
        BbsEntry {
            name: "ELITE FORCE BBS".into(),
            number: "555-3133".into(),
            sysop: "31337".into(),
            location: "Boston, MA".into(),
            baud: 14400,
            boards: vec!["Warez".into(), "Hacking".into(), "Music".into()],
            last_called: Some("04/01/93".into()),
            slug: "elite_force".into(),
            total_callers: 3_872,
            call_count: 31,
        },
        BbsEntry {
            name: "The Underground Railroad".into(),
            number: "555-7777".into(),
            sysop: "ConductorX".into(),
            location: "Chicago, IL".into(),
            baud: 2400,
            boards: vec!["Politics".into(), "News".into(), "General".into()],
            last_called: Some("03/28/93".into()),
            slug: "underground_railroad".into(),
            total_callers: 1_560,
            call_count: 5,
        },
        BbsEntry {
            name: "Midnight Rendezvous".into(),
            number: "555-2600".into(),
            sysop: "Pr0phet".into(),
            location: "New York, NY".into(),
            baud: 9600,
            boards: vec!["Phreaking".into(), "Underground".into(), "Lounge".into()],
            last_called: Some("03/14/93".into()),
            slug: "midnight_rendezvous".into(),
            total_callers: 2_047,
            call_count: 3,
        },
    ]
}
