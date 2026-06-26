# monkeytype-tui

> A monkeytype-style typing speed game that runs entirely in your terminal.

[![CI](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-2024_edition-orange)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

## Highlights

- 🖥️ **No browser, no Electron** — runs entirely in your terminal
- ⏱️ **Three modes** — Time (5s–custom), Words (10–500, custom), Quote
- 🌍 **48 languages** with word lists embedded at compile time; quotes in 38 languages
- 🎯 **Three difficulty levels** — Normal, Expert (any error resets), Master (space errors reset)
- 📊 **WPM, raw WPM, and accuracy** on every result screen, with personal best tracking
- 💾 **Persistent history** — last 50 results saved to `~/.local/share/monkeytype-tui/history.json`
- 🔍 **Language picker** with fuzzy search
- ⚙️ **Settings screen** — toggle sound, cursor shape, punctuation, numbers, and difficulty

## Overview

`monkeytype-tui` brings the [monkeytype](https://monkeytype.com) typing experience to the terminal. If you live in a terminal and want to practice typing without leaving it, this is for you.

It is built with [ratatui](https://github.com/ratatui-org/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm), with all word lists and quotes embedded at compile time — no network access, no runtime dependencies beyond the binary.

## Installation

**Requires:** Rust + Cargo ([install](https://rustup.rs)), a terminal with 256-color and Unicode support, and `libasound-dev` on Linux (for audio).

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run --release
```

## Usage

Launch the app and use `↑`/`↓` to pick a mode, then press `Enter` to start typing.

**Menu**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate |
| `←` / `→` | Change mode option |
| `1` / `2` / `3` | Switch to Time / Words / Quote mode |
| `Enter` | Start test |
| `p` / `n` | Toggle punctuation / numbers |
| `l` | Open language picker |
| `h` | Open history |
| `s` | Open settings |
| `?` | Open help |
| `Ctrl+C` / `q` | Quit |

**During a test**

| Key | Action |
|-----|--------|
| `Tab` | Restart |
| `Esc` | Back to menu |

**Result screen**

| Key | Action |
|-----|--------|
| `r` | Restart same test |
| `Enter` / `Tab` | New test (back to menu) |
| `Esc` | Back to menu |

## Supported Languages

Afrikaans, Albanian, Azerbaijani, Belarusian, Bosnian, Bulgarian, Catalan, Croatian, Czech, Danish, Dutch, English, Esperanto, Estonian, Filipino, Finnish, French, German, Greek, Hungarian, Icelandic, Indonesian, Irish, Italian, Japanese, Kazakh, Korean, Latin, Latvian, Lithuanian, Macedonian, Malay, Maltese, Mongolian, Norwegian, Polish, Portuguese, Romanian, Russian, Serbian, Slovak, Slovenian, Spanish, Swedish, Turkish, Ukrainian, Vietnamese, Welsh (48 total)

## Contributing

Bug reports, feature requests, and pull requests are welcome — open an issue to start the conversation.

**Adding a language:**

1. Add the word list JSON to `static/languages/`
2. Register a `lang!` entry in `src/words.rs`
3. If quotes exist, add the JSON to `static/quotes/` and add a branch in `src/words.rs`

**Development commands:**

```bash
cargo build       # compile
cargo run         # run
cargo test        # run tests
cargo clippy      # lint
```

## License

Licensed under [GPL-3.0](LICENSE).

Word lists (`static/languages/`) and quote collections (`static/quotes/`) are taken from [monkeytypegame/monkeytype](https://github.com/monkeytypegame/monkeytype) and distributed under the same license. No modifications were made to the data files. The game logic, UI, and Rust source code are original work unaffiliated with the monkeytype project.
