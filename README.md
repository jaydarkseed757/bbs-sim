# bbs-sim

A retro 1993 BBS simulator written in Rust with [macroquad](https://macroquad.rs/).

Dial into one of six bulletin board systems, log in, read message boards, send
private mail, and browse file libraries — all rendered in a green-phosphor
terminal aesthetic.

## Features

- **Phonebook dialer** — select a BBS and watch the modem handshake animate at the board's baud rate
- **Login** — banner text types out through the baud limiter; handle + password prompt
- **Message boards** — browse boards → thread list → read posts; compose new threads or replies
- **Private mail** — inbox with unread indicators; read, reply, and compose new messages
- **Files area** — per-BBS file libraries with sections (Shareware, Text Files, Utils, etc.); text files open in an inline viewer; binary files show a stub ZMODEM transfer
- **Logout** — goodbye message types out at modem speed, `NO CARRIER` on disconnect

## Running

```
cargo run
```

Requires a desktop environment (macroquad opens a native window).

## Controls

### Dialer
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate phonebook |
| `D` | Dial selected BBS |
| `Q` / `Esc` | Quit |

### Main Menu
| Key | Action |
|-----|--------|
| `B` | Message Boards |
| `M` | Mail |
| `F` | Files |
| `G` / `L` | Logout |

### Message Boards / Thread List
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate |
| `Enter` | Open selected item |
| `N` | New thread (in thread list) |
| `R` | Reply (in thread reader) |
| `↑` / `↓` / `PgUp` / `PgDn` | Scroll post |
| `F1` | Send composed message |
| `Esc` | Back |

### Mail
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate inbox |
| `Enter` | Read message |
| `R` | Reply |
| `C` | Compose new mail |
| `Tab` | Next field (compose) |
| `F1` | Send |
| `Esc` | Back |

### Files
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate file list |
| `V` | View text file inline |
| `D` | Download (stub ZMODEM) |
| `↑` / `↓` / `PgUp` / `PgDn` | Scroll file viewer |
| `Esc` | Back |

## Data

BBS content lives in `data/`:

```
data/
  bbs/          # per-BBS board/thread/post data  (<slug>.toml)
  mail/         # per-BBS private mailboxes       (<slug>.toml)
  files/        # per-BBS file libraries          (<slug>.toml)
```

Each BBS has a `slug` field in the hardcoded phonebook that maps to its TOML
files. Add new content by editing the TOML files — no code changes required.

## Project Structure

```
src/
  main.rs         # macroquad entry point
  app.rs          # App state, Screen enum, input/render/tick dispatch
  bbs/
    data.rs       # shared data types (Board, Thread, MailMessage, BbsFile, …)
    boards.rs     # board TOML loader
    mail.rs       # mail TOML loader
    files.rs      # files TOML loader
    session.rs    # login session state
  sim/
    modem.rs      # modem handshake simulation
    typer.rs      # baud-rate character throttle
  tui/
    boards.rs     # message board screens
    compose.rs    # compose new thread / reply
    dialer.rs     # phonebook screen
    dialing.rs    # modem dialing screen
    files.rs      # file list + inline text viewer
    login.rs      # login screen
    logout.rs     # goodbye / NO CARRIER screen
    mail.rs       # inbox, read mail, compose mail
    menus.rs      # main menu
    terminal.rs   # scrolling character grid (TerminalBuffer)
    ansi.rs       # ANSI escape handling
```

## Built With

- [macroquad](https://macroquad.rs/) — rendering
- [serde](https://serde.rs/) + [toml](https://docs.rs/toml) — data files
