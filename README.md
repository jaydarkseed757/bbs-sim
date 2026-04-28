# bbs-sim

A retro 1993 BBS simulator written in Rust with [macroquad](https://macroquad.rs/).

Dial into one of six bulletin board systems, log in, read message boards, send
private mail, browse file libraries, check the graffiti wall, and more — all
rendered in a green-phosphor terminal aesthetic at authentic baud rates.

## Features

- **Phonebook dialer** — select a BBS and watch the modem handshake animate at the board's baud rate; or press `[M]` to manually dial any number (always gets NO ANSWER)
- **Login** — banner types out through the baud limiter; handle + password prompt; call stats on login ("You are caller #1,847. You have called 14 times. Last on: 04/22/93.")
- **Message boards** — browse boards → thread list → read posts; compose new threads or replies
- **Private mail** — inbox with unread indicators; read, reply, and compose new messages
- **Files area** — per-BBS libraries with sections (Shareware, Text Files, Utils, etc.); text files open in an inline scrollable viewer; binary files show a stub ZMODEM transfer
- **Graffiti wall** — per-BBS one-liner list; scroll through entries, press `[A]` to leave your own; persists for the session
- **Top 10 lists** — three tabbed ranked lists: top callers, top downloads, top posters; gold/silver/bronze for the top 3
- **Chat with sysop** — pages the sysop by name at the board's baud rate; sysop is always away
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
| `M` | Manual dial — type any number |
| `Q` / `Esc` | Quit |

### Main Menu
| Key | Action |
|-----|--------|
| `B` | Message Boards |
| `M` | Mail |
| `F` | Files |
| `O` | One-liners / Graffiti Wall |
| `T` | Top 10 Lists |
| `C` | Chat with Sysop |
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

### Graffiti Wall
| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Scroll |
| `PgUp` / `PgDn` | Scroll by page |
| `A` | Add a one-liner |
| `Esc` | Main Menu |

### Top 10 Lists
| Key | Action |
|-----|--------|
| `Tab` or `1` / `2` / `3` | Switch between lists |
| `Esc` | Main Menu |

## Data

All BBS content is loaded from TOML files at login — no code changes needed to add or edit content.

```
data/
  bbs/          # per-BBS message boards, threads, posts   (<slug>.toml)
  mail/         # per-BBS private mailboxes                (<slug>.toml)
  files/        # per-BBS file libraries                   (<slug>.toml)
  oneliners/    # per-BBS graffiti wall entries            (<slug>.toml)
  top10/        # per-BBS top callers/downloads/posters    (<slug>.toml)
```

Each BBS has a `slug` in the hardcoded phonebook that maps to its TOML files.

## Project Structure

```
src/
  main.rs           # macroquad entry point
  app.rs            # App state, Screen enum, input/render/tick dispatch
  bbs/
    data.rs         # all data types (Board, MailMessage, BbsFile, Oneliner, TopLists, …)
    boards.rs       # board TOML loader
    files.rs        # files TOML loader
    mail.rs         # mail TOML loader
    oneliners.rs    # graffiti wall TOML loader
    top10.rs        # top 10 lists TOML loader
    session.rs      # login session state
  sim/
    modem.rs        # modem handshake + no-answer simulation
    typer.rs        # baud-rate character throttle
  tui/
    boards.rs       # message board screens
    compose.rs      # compose new thread / reply
    dialer.rs       # phonebook + manual dial overlay
    dialing.rs      # modem dialing screen
    files.rs        # file list + inline text viewer
    graffiti.rs     # graffiti wall screen
    login.rs        # login screen
    logout.rs       # goodbye / NO CARRIER screen
    mail.rs         # inbox, read mail, compose mail
    menus.rs        # main menu
    sysop.rs        # chat with sysop screen
    terminal.rs     # scrolling character grid (TerminalBuffer)
    top10.rs        # top 10 lists screen
    ansi.rs         # ANSI escape handling
```

## Built With

- [macroquad](https://macroquad.rs/) — rendering
- [serde](https://serde.rs/) + [toml](https://docs.rs/toml) — data files
