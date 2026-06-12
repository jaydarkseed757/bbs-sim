use macroquad::prelude::*;
use macroquad::audio::{play_sound, play_sound_once, Sound, PlaySoundParams};

use crate::bbs::banners::bbs_banner;
use crate::bbs::boards::{hardcoded_boards, load_boards};
use crate::bbs::data::{BbsEntry, Board, FileSection, MailMessage, Message, Oneliner, Thread, TopLists};
use crate::bbs::files::load_files;
use crate::bbs::mail::load_mail;
use crate::bbs::oneliners::load_oneliners;
use crate::bbs::session::Session;
use crate::bbs::top10::load_top10;
use crate::sim::modem::{DialPhase, ModemSim};
use crate::sim::typer::BaudTyper;
use crate::tui::crt::draw_crt_overlay;
use crate::tui::boards::{render_message_boards, render_read_thread, render_thread_list};
use crate::tui::compose::render_compose;
use crate::tui::download::render_download;
use crate::tui::files::{render_file_list, render_view_file};
use crate::tui::graffiti::render_graffiti_wall;
use crate::tui::mail::{render_compose_mail, render_inbox, render_read_mail};
use crate::tui::top10::render_top10;
use crate::tui::dialer::render_dialer;
use crate::tui::dialing::render_dialing;
use crate::tui::login::render_login;
use crate::tui::logout::render_logout;
use crate::tui::menus::render_main_menu;
use crate::tui::sysop::render_sysop_chat;
use crate::save::SaveData;
use crate::tui::font::{ch, BODY_PAD};
use crate::tui::terminal::{CellStyle, TerminalBuffer};
use crate::tui::theme::BbsTheme;

// Palette constants reused across tick methods.
const GREEN_BBS: Color    = Color::new(0.0,  0.85, 0.0,  1.0); // modem / dialing output
const WHITE_BBS: Color    = Color::new(0.85, 0.85, 0.85, 1.0); // login system text
const INPUT_GREEN: Color  = Color::new(0.0,  0.90, 0.0,  1.0); // user-typed chars

// ---------------------------------------------------------------------------
// Sounds
// ---------------------------------------------------------------------------

pub struct Sounds {
    pub handshake:    Sound,
    pub connect:      Sound,
    pub no_answer:    Sound,
    pub carrier_drop: Sound,
    pub nav_click:    Sound,
}

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
    DownloadFile,
    GraffitiWall,
    TopTen,
    SysopChat,
    Logout,
}

// ---------------------------------------------------------------------------
// Download state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadPhase {
    Negotiating { ticks: u32 },
    Transferring,
    Complete { elapsed_secs: f32 },
}

pub struct DownloadState {
    pub bbs_name:       String,
    pub file_name:      String,
    pub file_size_str:  String,
    pub file_size_bytes: u64,
    pub baud:           u32,
    pub bytes_done:     u64,
    pub phase:          DownloadPhase,
    pub cps_display:    f32,   // smoothed CPS shown on screen
    pub log_lines:      Vec<String>,
    pub tick_count:     u32,   // ticks spent in Transferring phase
}

impl DownloadState {
    pub fn new(bbs_name: String, file_name: String, file_size_str: String, baud: u32) -> Self {
        let file_size_bytes = parse_size(&file_size_str);
        let cps_display     = baud as f32 / 8.0 * 0.92; // theoretical ZMODEM efficiency
        Self {
            bbs_name,
            file_name,
            file_size_str,
            file_size_bytes,
            baud,
            bytes_done: 0,
            phase: DownloadPhase::Negotiating { ticks: 0 },
            cps_display,
            log_lines: vec!["ZRQINIT...".into()],
            tick_count: 0,
        }
    }

    /// Wall-clock seconds elapsed in the Transferring phase (for display).
    pub fn elapsed_secs(&self) -> f32 {
        self.tick_count as f32 / 20.0
    }
}

/// Parse size strings like "1.4M", "320K", "8K" into bytes.
fn parse_size(s: &str) -> u64 {
    let s = s.trim();
    if let Some(num) = s.strip_suffix('M') {
        (num.parse::<f32>().unwrap_or(0.0) * 1024.0 * 1024.0) as u64
    } else if let Some(num) = s.strip_suffix('K') {
        (num.parse::<f32>().unwrap_or(0.0) * 1024.0) as u64
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
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
        let modem = ModemSim::new(&bbs.number, bbs.baud);
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

        let banner = bbs_banner(&bbs);
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
    pub sounds: Option<Sounds>,
    pub blink: bool,
    blink_ticks: u32,
    pub tick_count: u64,
    pub frame_count: u64,
    pub sessions_this_run: u32,
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
    pub file_download_notice: bool,
    pub download: Option<DownloadState>,
    pub manual_dial_input: Option<String>,
    pub oneliners: Vec<Oneliner>,
    pub graffiti_scroll: usize,
    pub graffiti_input: Option<String>,
    pub top_lists: TopLists,
    pub top_list_tab: usize,
    pub theme: BbsTheme,
    pub save: SaveData,
}

impl App {
    pub fn new() -> Self {
        let save     = SaveData::load();
        let phonebook = save.apply_to_phonebook(hardcoded_phonebook());
        Self {
            screen: Screen::Dialer,
            should_quit: false,
            sounds: None,
            blink: true,
            blink_ticks: 0,
            tick_count: 0,
            frame_count: 0,
            sessions_this_run: 0,
            phonebook,
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
            file_download_notice: false,
            download: None,
            manual_dial_input: None,
            oneliners: vec![],
            graffiti_scroll: 0,
            graffiti_input: None,
            top_lists: TopLists::default(),
            top_list_tab: 0,
            theme: BbsTheme::default(),
            save,
        }
    }

    pub fn render(&mut self) {
        self.frame_count += 1;
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
            Screen::ComposeMessage => render_compose(self),
            Screen::Mail          => render_inbox(self),
            Screen::ReadMail { id } => render_read_mail(self, id),
            Screen::ComposeMail   => render_compose_mail(self),
            Screen::Files              => render_file_list(self),
            Screen::ViewFile { id }    => render_view_file(self, id),
            Screen::DownloadFile => render_download(self),
            Screen::GraffitiWall    => render_graffiti_wall(self),
            Screen::TopTen          => render_top10(self),
            Screen::SysopChat       => render_sysop_chat(self),
            Screen::Logout        => render_logout(self),
        }
        draw_crt_overlay();
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
            Screen::ComposeMessage => self.compose_input(),
            Screen::Mail          => self.inbox_input(),
            Screen::ReadMail { id } => self.read_mail_input(id),
            Screen::ComposeMail   => self.compose_mail_input(),
            Screen::Files              => self.files_input(),
            Screen::ViewFile { id }    => self.view_file_input(id),
            Screen::DownloadFile => self.download_input(),
            Screen::GraffitiWall    => self.graffiti_wall_input(),
            Screen::TopTen          => self.top10_input(),
            Screen::SysopChat       => self.sysop_chat_input(),
            Screen::Logout        => self.logout_input(),
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        self.blink_ticks += 1;
        if self.blink_ticks >= 10 {
            self.blink = !self.blink;
            self.blink_ticks = 0;
        }

        match self.screen.clone() {
            Screen::Dialing      => self.tick_dialing(),
            Screen::Login        => self.tick_login(),
            Screen::Logout       => self.tick_logout(),
            Screen::SysopChat    => self.tick_sysop_chat(),
            Screen::DownloadFile => self.tick_download(),
            _                    => {}
        }
    }

    // ── Tick: dialing ────────────────────────────────────────────────────────

    fn tick_dialing(&mut self) {
        let Some(ref mut d) = self.dial else { return };

        let prev_phase = d.modem.phase.clone();

        if let Some(text) = d.modem.tick() {
            d.typer.enqueue(&text);
        }
        for ch in d.typer.tick() {
            d.buffer.push_char(ch, CellStyle::fg(GREEN_BBS));
        }

        // Sound on phase transitions.
        if d.modem.phase != prev_phase {
            if let Some(ref sounds) = self.sounds {
                match (&prev_phase, &d.modem.phase) {
                    (DialPhase::Dialing, DialPhase::Connecting) => {
                        play_sound(&sounds.handshake, PlaySoundParams { looped: false, volume: 0.55 });
                    }
                    (_, DialPhase::Connected) => {
                        play_sound_once(&sounds.connect);
                    }
                    (_, DialPhase::NoAnswer) => {
                        play_sound_once(&sounds.no_answer);
                    }
                    _ => {}
                }
            }
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
            self.sessions_this_run += 1;
            self.theme = BbsTheme::for_slug(&slug);

            // Use saved BBS data if available; otherwise load fresh from TOML.
            if let Some(saved) = self.save.bbs.get(&slug).cloned() {
                self.boards    = saved.boards;
                self.mail      = saved.mail;
                self.oneliners = saved.oneliners;
                // Mail was personalized for the handle it was saved under;
                // remap to/from if this session uses a different one.
                if !saved.handle.is_empty() && saved.handle != handle {
                    for m in &mut self.mail {
                        if m.to   == saved.handle { m.to   = handle.clone(); }
                        if m.from == saved.handle { m.from = handle.clone(); }
                    }
                }
            } else {
                self.boards    = load_boards(&slug);
                self.mail      = load_mail(&slug, &handle);
                self.oneliners = load_oneliners(&slug);
            }
            self.graffiti_scroll = self.oneliners.len().saturating_sub(1);
            self.files      = load_files(&slug);   // files are read-only; always load fresh
            self.top_lists  = load_top10(&slug);
            self.top_list_tab        = 0;
            self.selected_board_row  = 0;
            self.selected_thread_row = 0;
            self.selected_mail_row   = 0;
            self.selected_file_row   = 0;
            self.read_scroll         = 0;
            self.mail_read_scroll    = 0;
            self.file_view_scroll    = 0;
            if let Some(entry) = self.phonebook.iter_mut().find(|e| e.name == bbs_name) {
                entry.call_count  += 1;
                entry.last_called  = Some("04/28/93".into());
            }
            self.persist_session();
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
                    // A manually dialed number that matches a phonebook entry
                    // connects normally; anything else rings out.
                    let digits = |s: &str| s.chars().filter(char::is_ascii_digit).collect::<String>();
                    let typed = digits(&number);
                    let entry = self.phonebook.iter()
                        .find(|e| digits(&e.number) == typed)
                        .cloned();
                    self.dial = Some(match entry {
                        Some(bbs) => DialingState::new(bbs),
                        None      => DialingState::new_manual(number),
                    });
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
            if let Some(ref s) = self.sounds { play_sound_once(&s.nav_click); }
        }
        if is_key_pressed(KeyCode::Down) {
            self.selected_row =
                if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
            if let Some(ref s) = self.sounds { play_sound_once(&s.nav_click); }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.save.capture_phonebook(&self.phonebook.clone());
            self.save.write();
            self.should_quit = true;
        }

        while let Some(ch) = get_char_pressed() {
            match ch {
                'q' | 'Q' => {
                    self.save.capture_phonebook(&self.phonebook.clone());
                    self.save.write();
                    self.should_quit = true;
                }
                'j' => {
                    self.selected_row =
                        if self.selected_row + 1 >= len { 0 } else { self.selected_row + 1 };
                    if let Some(ref s) = self.sounds { play_sound_once(&s.nav_click); }
                }
                'k' => {
                    self.selected_row =
                        if self.selected_row == 0 { len.saturating_sub(1) } else { self.selected_row - 1 };
                    if let Some(ref s) = self.sounds { play_sound_once(&s.nav_click); }
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
                    self.file_download_notice = false;
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
            match ch.to_ascii_lowercase() {
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
            match ch.to_ascii_lowercase() {
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
        {
            use crate::tui::boards::{READ_FOOTER_CH, READ_HEADER_CH};
            let body_h = screen_height() - ch() * (READ_HEADER_CH + READ_FOOTER_CH);
            let rows_vis = ((body_h - BODY_PAD) / ch()) as usize;
            let max_scroll = self.thread_line_count(&board_id, thread_id).saturating_sub(rows_vis);
            self.read_scroll = self.read_scroll.min(max_scroll);
        }

        while let Some(ch) = get_char_pressed() {
            if ch.eq_ignore_ascii_case(&'r') {
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

    /// Count display lines for a thread, matching the layout in `render_read_thread`.
    fn thread_line_count(&self, board_id: &str, thread_id: u32) -> usize {
        let thread = self.boards.iter()
            .find(|b| b.id == board_id)
            .and_then(|b| b.threads.iter().find(|t| t.id == thread_id));
        let Some(thread) = thread else { return 0 };
        let mut count = 0;
        for (i, msg) in thread.posts.iter().enumerate() {
            if i > 0 { count += 2; }        // blank line + separator row
            count += 3;                      // From header, Subj header, blank line
            count += msg.body.replace('\r', "").split('\n').count();
        }
        count
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
        self.persist_session();
    }

    // ── Input: files ─────────────────────────────────────────────────────────

    fn files_input(&mut self) {
        let any_key = is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::Escape);

        if self.file_download_notice && any_key {
            self.file_download_notice = false;
        }

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
            if self.file_download_notice {
                self.file_download_notice = false;
                continue;
            }
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
                        let bbs_name = self.session.bbs_name.clone().unwrap_or_default();
                        let baud = self.phonebook.iter()
                            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
                            .map(|e| e.baud)
                            .unwrap_or(2400);
                        self.download = Some(DownloadState::new(
                            bbs_name, f.name, f.size, baud,
                        ));
                        self.screen = Screen::DownloadFile;
                    }
                }
                _ => {}
            }
        }
    }

    fn view_file_input(&mut self, id: u32) {
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
        if let Some(file) = self.files.iter().flat_map(|s| s.files.iter()).find(|f| f.id == id) {
            use crate::tui::files::{VIEW_FOOTER_CH, VIEW_HEADER_CH};
            let body_h = screen_height() - ch() * (VIEW_HEADER_CH + VIEW_FOOTER_CH);
            let rows_vis = ((body_h - BODY_PAD) / ch()) as usize;
            let max_scroll = file.content.split('\n').count().saturating_sub(rows_vis);
            self.file_view_scroll = self.file_view_scroll.min(max_scroll);
        }
    }

    // ── Input / tick: download ───────────────────────────────────────────────

    fn download_input(&mut self) {
        let done = self.download.as_ref()
            .map(|d| matches!(d.phase, DownloadPhase::Complete { .. }))
            .unwrap_or(false);

        if done {
            // Any key returns to file list
            let any = get_char_pressed().is_some()
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Escape);
            if any {
                self.download = None;
                self.screen   = Screen::Files;
            }
        } else if is_key_pressed(KeyCode::Escape) {
            // Cancel — abort transfer
            self.download = None;
            self.screen   = Screen::Files;
        }
    }

    fn tick_download(&mut self) {
        let dl = match self.download.as_mut() { Some(d) => d, None => return };

        // Speed multiplier: sim runs ~15× real ZMODEM speed for playability.
        // bytes_per_tick = (baud_bps / 8) × efficiency × speed_factor / 20_Hz
        const SPEED_FACTOR: f32 = 15.0;
        const EFFICIENCY:   f32 = 0.92;
        let bytes_per_sec_real = dl.baud as f32 / 8.0 * EFFICIENCY;
        let bytes_per_tick_avg = (bytes_per_sec_real * SPEED_FACTOR / 20.0) as u64;

        match dl.phase.clone() {
            DownloadPhase::Negotiating { ticks } => {
                let t = ticks + 1;
                // ZMODEM handshake log lines appear progressively
                match t {
                    5  => dl.log_lines.push("ZRINIT...".into()),
                    12 => dl.log_lines.push(format!("ZFILE {}...", dl.file_name)),
                    18 => dl.log_lines.push(format!("ZDATA 0x{:08X}", 0u32)),
                    25 => dl.log_lines.push("Receiving data blocks...".into()),
                    _  => {}
                }
                if t >= 30 {
                    dl.phase = DownloadPhase::Transferring;
                } else {
                    dl.phase = DownloadPhase::Negotiating { ticks: t };
                }
            }

            DownloadPhase::Transferring => {
                // Add mild jitter (±15%) to CPS display
                let jitter = {
                    let t = dl.tick_count;
                    let wave = (t as f32 * 0.23).sin() * 0.10
                             + (t as f32 * 0.07).sin() * 0.05;
                    1.0_f32 + wave
                };
                let bytes_this_tick = ((bytes_per_tick_avg as f32) * jitter) as u64;
                dl.bytes_done = (dl.bytes_done + bytes_this_tick).min(dl.file_size_bytes);
                dl.tick_count += 1;

                // Smooth CPS toward the current instantaneous rate
                let instant_cps = bytes_per_sec_real * SPEED_FACTOR * jitter;
                dl.cps_display += (instant_cps - dl.cps_display) * 0.15;

                // Protocol chatter: emit a log line every ~8 ticks
                if dl.tick_count % 8 == 0 {
                    let block = dl.bytes_done / 1024;
                    dl.log_lines.push(format!(
                        "ZDATA 0x{:08X}  ({} KB)",
                        dl.bytes_done as u32, block
                    ));
                }
                // Occasional noise: error-recovery lines
                if dl.tick_count % 53 == 0 {
                    dl.log_lines.push("ZRPOS  (resync)".into());
                }
                if dl.tick_count % 97 == 0 {
                    dl.log_lines.push(format!(
                        "ZNAK  block {}  (retry)", dl.tick_count / 97
                    ));
                }

                if dl.bytes_done >= dl.file_size_bytes {
                    let elapsed = dl.elapsed_secs();
                    dl.phase = DownloadPhase::Complete { elapsed_secs: elapsed };
                    dl.log_lines.push("ZEOF...".into());
                    dl.log_lines.push("ZFIN...".into());
                    dl.log_lines.push(format!(
                        "Transfer complete.  {} bytes in {}",
                        dl.file_size_bytes,
                        fmt_elapsed(elapsed),
                    ));
                }
            }

            DownloadPhase::Complete { .. } => {}
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
            match ch.to_ascii_lowercase() {
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
        if let Some(msg) = self.mail.iter().find(|m| m.id == id) {
            use crate::tui::mail::{READ_FOOTER_CH, READ_HEADER_CH};
            let body_h = screen_height() - ch() * (READ_HEADER_CH + READ_FOOTER_CH);
            let rows_vis = ((body_h - BODY_PAD) / ch()) as usize;
            let max_scroll = msg.body.split('\n').count().saturating_sub(rows_vis);
            self.mail_read_scroll = self.mail_read_scroll.min(max_scroll);
        }

        while let Some(ch) = get_char_pressed() {
            if ch.eq_ignore_ascii_case(&'r') {
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
        self.persist_session();
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
                    self.persist_session();
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
        let green = self.theme.hi;
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
        let green = self.theme.hi;
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
        if let Some(ref sounds) = self.sounds {
            play_sound_once(&sounds.carrier_drop);
        }
        let bbs_name = self.session.bbs_name.clone().unwrap_or_default();
        let handle   = self.session.user_handle.clone().unwrap_or_default();
        let baud     = self.phonebook.iter()
            .find(|e| Some(&e.name) == self.session.bbs_name.as_ref())
            .map(|e| e.baud)
            .unwrap_or(2400);
        self.logout = Some(LogoutState::new(bbs_name, &handle, baud));
        self.screen = Screen::Logout;
    }

    /// Snapshot the live session (boards/mail/oneliners + phonebook) into the
    /// save file. Called after every user mutation so a window close can't
    /// lose data — exit paths are not guaranteed to run under macroquad.
    fn persist_session(&mut self) {
        let slug = self.session.bbs_name.as_ref()
            .and_then(|n| self.phonebook.iter().find(|e| &e.name == n))
            .map(|e| e.slug.clone())
            .unwrap_or_default();
        let handle = self.session.user_handle.clone().unwrap_or_default();
        self.save.capture_bbs(
            &slug,
            &handle,
            self.boards.clone(),
            self.mail.clone(),
            self.oneliners.clone(),
        );
        self.save.capture_phonebook(&self.phonebook.clone());
        self.save.write();
    }

    fn finish_logout(&mut self) {
        if let Some(ref state) = self.logout {
            let bbs_name = state.bbs_name.clone();
            if let Some(entry) = self.phonebook.iter_mut().find(|e| e.name == bbs_name) {
                entry.last_called = Some("04/27/93".into());
            }
        }

        self.persist_session();

        self.session.logout();
        self.logout   = None;

        // Clear all per-session data so nothing leaks into the next dial.
        self.boards              = vec![];
        self.mail                = vec![];
        self.files               = vec![];
        self.oneliners           = vec![];
        self.top_lists           = TopLists::default();
        self.compose             = None;
        self.compose_mail        = None;
        self.download            = None;
        self.graffiti_input      = None;
        self.selected_board_row  = 0;
        self.selected_thread_row = 0;
        self.selected_mail_row   = 0;
        self.selected_file_row   = 0;
        self.read_scroll         = 0;
        self.mail_read_scroll    = 0;
        self.file_view_scroll    = 0;
        self.graffiti_scroll     = 0;
        self.top_list_tab        = 0;
        self.file_download_notice = false;
        self.theme               = BbsTheme::default();

        self.screen = Screen::Dialer;
    }

    // ── Exit stats ───────────────────────────────────────────────────────────

    pub fn print_exit_stats(&self) {
        let runtime_secs  = self.tick_count as f64 / 20.0;
        let runtime_min   = runtime_secs as u64 / 60;
        let runtime_sec   = runtime_secs as u64 % 60;
        let avg_fps       = if runtime_secs > 0.0 { self.frame_count as f64 / runtime_secs } else { 0.0 };

        let pb_total      = self.phonebook.len();
        let pb_called     = self.phonebook.iter().filter(|e| e.call_count > 0).count();

        let board_count   = self.boards.len();
        let thread_count: usize = self.boards.iter().map(|b| b.threads.len()).sum();
        let post_count:   usize = self.boards.iter()
            .flat_map(|b| b.threads.iter())
            .map(|t| t.posts.len())
            .sum();
        let mail_count    = self.mail.len();
        let sect_count    = self.files.len();
        let file_count: usize = self.files.iter().map(|s| s.files.len()).sum();
        let oneliner_count = self.oneliners.len();
        let top_callers   = self.top_lists.callers.len();
        let top_files     = self.top_lists.files.len();
        let top_posters   = self.top_lists.posters.len();

        let last_bbs    = self.session.bbs_name.as_deref()
            .or_else(|| self.phonebook.iter()
                .filter(|e| e.last_called.is_some())
                .max_by_key(|e| e.call_count)
                .map(|e| e.name.as_str()))
            .unwrap_or("—");
        let last_handle = self.session.user_handle.as_deref().unwrap_or("—");

        let w = 57;
        let bar = "=".repeat(w);
        println!();
        println!("{}", bar);
        println!("  BBS-SIM  —  exit stats");
        println!("{}", bar);
        println!("  runtime          {}m {:02}s   ({} ticks / {} frames)",
            runtime_min, runtime_sec,
            fmt_u64(self.tick_count), fmt_u64(self.frame_count));
        println!("  avg fps          {:.1}", avg_fps);
        println!("  sessions this run  {}", self.sessions_this_run);
        println!();
        println!("  phonebook        {} entries  ({} ever called)",
            pb_total, pb_called);
        println!("  last BBS         {}", last_bbs);
        println!("  last handle      {}", last_handle);
        println!();
        println!("  content (last session loaded)");
        if board_count > 0 {
            println!("    boards         {}  |  threads {}  |  posts {}", board_count, thread_count, post_count);
        } else {
            println!("    boards         (none loaded)");
        }
        println!("    mail           {}", if mail_count > 0 { mail_count.to_string() } else { "(none)".into() });
        if sect_count > 0 {
            println!("    files          {} sections  /  {} files", sect_count, file_count);
        } else {
            println!("    files          (none loaded)");
        }
        println!("    one-liners     {}", if oneliner_count > 0 { oneliner_count.to_string() } else { "(none)".into() });
        if top_callers + top_files + top_posters > 0 {
            println!("    top-10         callers {}  /  files {}  /  posters {}", top_callers, top_files, top_posters);
        }
        println!("{}", bar);
        println!();
    }
}

fn fmt_u64(n: u64) -> String {
    // comma-format a u64
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
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
        BbsEntry {
            name: "FREQ 867 BBS".into(),
            number: "867-5309".into(),
            sysop: "SpinDr".into(),
            location: "Nashville, TN".into(),
            baud: 14400,
            boards: vec!["Alt/Grunge".into(), "Metal".into(), "Hip-Hop".into(), "New Releases".into(), "Concert Talk".into()],
            last_called: None,
            slug: "freq_867".into(),
            total_callers: 3_847,
            call_count: 0,
        },
    ]
}

fn fmt_elapsed(secs: f32) -> String {
    let s  = secs as u32;
    let m  = s / 60;
    let ss = s % 60;
    if m >= 60 {
        format!("{}h {:02}m {:02}s", m / 60, m % 60, ss)
    } else {
        format!("{}:{:02}", m, ss)
    }
}
