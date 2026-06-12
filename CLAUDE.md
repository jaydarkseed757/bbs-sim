# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # compile
cargo run            # build and launch the native window
cargo check          # fast type-check without producing a binary
cargo clippy         # lints (target: zero warnings)
```

There are no tests and no separate lint step beyond `clippy`. The project requires a desktop environment — macroquad opens a native window on run.

## Architecture

### Main loop (`src/main.rs`)

macroquad drives a fixed 20 Hz tick loop:

```
every frame:  handle_input() → tick() (if 50 ms elapsed) → render()
```

`App::tick_count` / `App::frame_count` track lifetime totals. On `app.should_quit`, the loop breaks and `app.print_exit_stats()` prints runtime info to stdout. Note: code after the loop does NOT run when the user closes the window — never rely on exit paths for persistence (see Save system).

### Central coordinator (`src/app.rs`)

Everything lives on `App`. The `Screen` enum is the state machine; `handle_input`, `tick`, and `render` all dispatch on the current screen variant. Input handlers are private methods (`fn message_boards_input`, `fn read_thread_input`, etc.) called from `handle_input`. Tick methods are similarly named (`fn tick_login`, `fn tick_dialing`, etc.).

Key `App` fields:
- `screen: Screen` — current state
- `theme: BbsTheme` — set from the BBS slug at login, reset to default at logout
- `boards / mail / files / oneliners / top_lists` — loaded at login (from save if present, else TOML)
- `session: Session` — holds `user_handle` and `bbs_name` while logged in
- `save: SaveData` — loaded from `save.json` in `App::new()`
- `phonebook: Vec<BbsEntry>` — hardcoded in `hardcoded_phonebook()` at the bottom of `app.rs`, with saved call counts applied on top
- Scroll positions (`read_scroll`, `mail_read_scroll`, `file_view_scroll`) are stored on `App` and clamped in the input handlers using chrome-height constants exported by the render modules (`READ_HEADER_CH`/`READ_FOOTER_CH` in `tui/boards.rs` and `tui/mail.rs`, `VIEW_HEADER_CH`/`VIEW_FOOTER_CH` in `tui/files.rs`, shared `BODY_PAD` in `tui/font.rs`) — renderers use the same constants so the two can't drift

Manual dialing (M on the dialer) normalizes the typed number to digits and matches it against the phonebook; matches connect normally, unknown numbers get NO ANSWER via `DialingState::new_manual`.

### Save system (`src/save.rs`)

Game state persists to `save.json` (pretty JSON, working directory):
- `SaveData` holds a `schema` version, per-BBS-name phonebook stats (`call_count`, `last_called`), and a `HashMap<slug, BbsSave>` of full boards/mail/oneliners state
- `BbsSave` also stores the `handle` it was saved under; at login-restore, mail `to`/`from` matching the old handle is remapped to the new one
- **Saving happens at mutation time**, not on exit: `App::persist_session()` is called after every post (`do_post`), sent mail (`do_send_mail`), graffiti line, and at login (after the call-count bump). `finish_logout` delegates to it. The dialer quit path saves phonebook only.
- Missing/corrupt/schema-mismatched save files fall back to defaults with a stderr note — never panic
- At login, if `save.bbs` has the slug, boards/mail/oneliners come from the save; otherwise from TOML. `files` and `top_lists` always load fresh from TOML (read-only content).

### Three sub-modules (`src/bbs/`, `src/sim/`, `src/tui/`)

**`src/bbs/`** — data layer
- `data.rs`: all shared structs (`BbsEntry`, `Board`, `Thread`, `Message`, `MailMessage`, `BbsFile`, `FileSection`, `TopLists`, etc.) — all derive Serialize/Deserialize
- Loaders (`boards.rs`, `mail.rs`, `files.rs`, `oneliners.rs`, `top10.rs`): each reads `data/<category>/<slug>.toml` via serde + toml; returns empty Vec on missing/malformed file. `load_mail` personalizes each message's `to` field with the session handle.
- `banners.rs`: per-slug ASCII art, dispatched by `bbs_banner(bbs)`
- `session.rs`: thin `Session` struct (login/logout, no persistence)

**`src/sim/`** — simulation
- `modem.rs`: `ModemSim` steps through `DialPhase` states emitting AT command strings each tick; carries the BBS `baud` so the connect string reads `CONNECT {baud}`
- `typer.rs`: `BaudTyper` throttles a char queue to the BBS baud rate at 20 Hz tick rate (characters-per-tick or ticks-per-character depending on baud)
- `audio.rs`: WAV bytes baked in at compile time for modem sounds

**`src/tui/`** — rendering
- One file per screen (`dialer.rs`, `dialing.rs`, `login.rs`, `menus.rs`, `boards.rs`, `compose.rs`, `mail.rs`, `files.rs`, `graffiti.rs`, `top10.rs`, `download.rs`, `logout.rs`, `sysop.rs`)
- `font.rs`: `ch()` (line height), `cw()` (char width), `s()` (identity px pass-through), `txt()`, `tw()` (text width measure), `BODY_PAD` — import these instead of raw macroquad calls
- `terminal.rs`: `TerminalBuffer` — scrolling per-character color grid used for animated dialing/login/logout/sysop screens; `push_char` handles `\r`/`\n`; `visible_lines(n)` returns the last n rows
- `theme.rs`: `BbsTheme` with 10 color fields; `BbsTheme::for_slug(slug)` returns the palette
- `crt.rs`: scanline overlay drawn on top of every frame

### Theme system

All render functions access colors via `let t = &app.theme;` and use `t.title`, `t.hi`, `t.lo`, `t.label`, `t.body`, `t.muted`, `t.border`, `t.sel_bg`, `t.cursor`, `t.stat`. Never declare local color constants in render modules (exceptions: `top10.rs` medal colors, `crt.rs` overlay). The animated terminal screens (logout, sysop) use `app.theme.hi` for typed-out text and `t.border`/`t.title`/`t.muted` for chrome. The pre-login screens (dialer, dialing, login) run while `app.theme` is still the default palette, so their colors effectively are the default theme.

### Data files

```
data/
  bbs/<slug>.toml         # boards → threads → posts
  mail/<slug>.toml        # [[messages]] with from/to/subject/body/timestamp
  files/<slug>.toml       # [[sections]] each with [[sections.files]]
  oneliners/<slug>.toml   # [[lines]] with handle/message/date
  top10/<slug>.toml       # [[callers]], [[files]], [[posters]]
```

Content is period-accurate to 1993. Adding or editing content only requires TOML edits — no code changes. Note: once a BBS has an entry in `save.json`, its boards/mail/oneliners load from the save, so TOML edits to those categories won't appear until the save entry (or file) is deleted.

### Adding a new BBS

1. Add a `BbsEntry` to `hardcoded_phonebook()` in `app.rs` with a unique `slug`
2. Create `data/{bbs,mail,files,oneliners,top10}/<slug>.toml`
3. Add a banner function in `src/bbs/banners.rs` and wire it in `bbs_banner()`
4. Add a palette constructor in `src/tui/theme.rs` and wire it in `BbsTheme::for_slug()`
