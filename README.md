# bbs-sim

A retro 1993 BBS simulator written in Rust with [macroquad](https://macroquad.rs/).

Dial into one of seven bulletin board systems, log in, read message boards, send
private mail, browse file libraries, sign the graffiti wall, and check the Top 10
lists — each board rendered in its own period color palette under a CRT scanline
overlay.

## Features

- **Phonebook dialer** — select a BBS and watch the modem handshake animate at the board's baud rate, or manually dial any number (known numbers connect; unknown ones ring out with NO ANSWER)
- **Login** — per-BBS ANSI or ASCII art banner; handle + password prompt; call stats on login
- **Per-BBS themes** — each board has its own color palette (green phosphor, LCARS blue, amber, hacker red, purple, hot orange…) applied across every screen
- **Message boards** — browse boards → thread list → read posts; compose new threads or replies
- **Private mail** — inbox with unread indicators; read, reply, and compose new messages
- **Files area** — per-BBS file libraries with sections; text files open in an inline viewer; binary files run a simulated ZMODEM transfer with progress bar and CPS readout
- **Graffiti wall** — scrolling one-liner wall; add your own line
- **Top 10 lists** — top callers, downloads, and posters per BBS
- **Voting booth** — yes/no polls with bar-chart results once you've voted
- **Last callers** — recent-caller log per BBS ([W]ho's been on)
- **Door game** — The Gauntlet: pick a class, battle escalating monsters, chase a high score
- **Sysop chat** — page the sysop (they're never around)
- **Save game** — posts, sent mail, graffiti lines, and phonebook call history persist to `save.json`; written after every change so nothing is lost on window close
- **Logout** — goodbye message types out at modem speed, `NO CARRIER` on disconnect
- **Exit stats** — runtime, fps, and content totals print to the console on quit

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
| `M` | Manual dial (type any number) |
| `Q` / `Esc` | Quit |

### Main Menu
| Key | Action |
|-----|--------|
| `B` | Message Boards |
| `M` | Mail |
| `F` | Files |
| `O` | Graffiti Wall (one-liners) |
| `T` | Top 10 Lists |
| `V` | Voting Booth |
| `W` | Last Callers |
| `D` | Door Game (The Gauntlet) |
| `C` | Chat with Sysop |
| `G` / `L` / `Esc` | Logout |

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
| `D` | Download (simulated ZMODEM) |
| `↑` / `↓` / `PgUp` / `PgDn` | Scroll file viewer |
| `Esc` | Back |

### Graffiti Wall
| Key | Action |
|-----|--------|
| `↑` / `↓` / `PgUp` / `PgDn` | Scroll wall |
| `A` | Add a line |
| `Enter` | Post line |
| `Esc` | Back |

### Top 10
| Key | Action |
|-----|--------|
| `Tab` or `1` / `2` / `3` | Switch list (callers / files / posters) |
| `Esc` | Back |

## Data

BBS content lives in `data/`, one TOML file per BBS slug per category:

```
data/
  bbs/          # boards → threads → posts
  mail/         # private mailboxes
  files/        # file libraries (sections + files)
  oneliners/    # graffiti wall lines
  top10/        # top callers / files / posters
  voting/       # yes/no polls
  callers/      # last-callers log
  banners/      # ANSI art login banners (<slug>.ans)
```

Each BBS has a `slug` field in the hardcoded phonebook that maps to its TOML
files. Add new content by editing the TOML files — no code changes required.
Content is period-accurate to 1993.

Game state persists to `save.json` in the working directory. Delete it to
start fresh.

## Project Structure

```
src/
  main.rs         # macroquad entry point, 20 Hz tick loop
  app.rs          # App state, Screen enum, input/render/tick dispatch, phonebook
  save.rs         # save.json persistence (SaveData, BbsSave)
  bbs/
    data.rs       # shared data types (Board, Thread, MailMessage, BbsFile, …)
    boards.rs     # board TOML loader
    mail.rs       # mail TOML loader
    files.rs      # files TOML loader
    oneliners.rs  # graffiti wall TOML loader
    top10.rs      # top-10 lists TOML loader
    voting.rs     # polls TOML loader
    callers.rs    # last-callers TOML loader
    banner.rs     # ANSI art banner file loader
    banners.rs    # per-BBS ASCII art login banners (fallback)
    session.rs    # login session state
  sim/
    modem.rs      # modem handshake simulation (AT commands, line noise)
    typer.rs      # baud-rate character throttle
    audio.rs      # compile-time generated modem sound effects
  tui/
    theme.rs      # per-BBS color palettes (BbsTheme)
    font.rs       # font + layout helpers (ch, cw, txt, tw)
    terminal.rs   # scrolling character grid (TerminalBuffer)
    crt.rs        # CRT scanline overlay
    dialer.rs     # phonebook screen
    dialing.rs    # modem dialing screen
    login.rs      # login screen
    menus.rs      # main menu
    boards.rs     # message board screens
    compose.rs    # compose new thread / reply
    mail.rs       # inbox, read mail, compose mail
    files.rs      # file list + inline text viewer
    download.rs   # simulated ZMODEM transfer screen
    graffiti.rs   # graffiti wall
    top10.rs      # top-10 lists
    voting.rs     # voting booth
    callers.rs    # last-callers log
    door.rs       # door game (The Gauntlet)
    sysop.rs      # sysop chat screen
    logout.rs     # goodbye / NO CARRIER screen
    ansi.rs       # ANSI escape handling
```

## Built With

- [macroquad](https://macroquad.rs/) — rendering and audio
- [serde](https://serde.rs/) + [toml](https://docs.rs/toml) — data files
- [serde_json](https://docs.rs/serde_json) — save file
