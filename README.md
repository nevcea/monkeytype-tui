# monkeytype-tui

> A monkeytype-style typing speed game that runs entirely in your terminal.

[![CI](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-2024_edition-orange)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

## Highlights

- 🖥️ **No browser, no Electron** — runs entirely in your terminal
- ⏱️ **Three modes** — Time (15/30/60/120s, custom up to 3600s), Words (10/25/50/100, custom up to 5000), Quote (with a length filter: short/medium/long/thicc)
- 🌍 **48 languages** with word lists embedded at compile time; quotes in 37 languages
- 🎯 **Three difficulty levels** — Normal, Expert (any mistake fails the test), Master (fails if the word you just finished had a mistake in it); any run also fails below 75% accuracy
- 📊 **WPM, raw WPM, and accuracy** on every result screen, with personal best tracking
- 💾 **Persistent history** — last 50 results saved to `~/.local/share/monkeytype-tui/history.json`
- 🔍 **Language picker** with fuzzy search
- ⚙️ **Settings screen** — cursor shape, sound & volume, history expiry, and difficulty (punctuation/numbers toggle from the menu instead)

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

Launch the app, use `1`/`2`/`3` to pick Time / Words / Quote, `←`/`→` to change the option (or cycle the quote length filter in Quote mode), then press `Enter`/`Tab` to start typing.

**Menu**

| Key | Action |
|-----|--------|
| `1` / `2` / `3` | Switch to Time / Words / Quote mode |
| `←` / `→` | Change mode option (or quote length filter in Quote mode) |
| `Enter` / `Tab` | Start test (opens a custom number prompt if the "custom" slot is selected) |
| `p` / `n` | Toggle punctuation / numbers |
| `l` | Open language picker (fuzzy search, `←`/`→` change word-pool size) |
| `h` | Open history |
| `s` | Open settings |
| `?` | Open help |
| `Ctrl+C` | Quit immediately |
| `q` / `Esc` | Quit (asks for confirmation) |

**During a test**

| Key | Action |
|-----|--------|
| `Tab` | Restart with new words |
| `Backspace` | Delete last character |
| `Ctrl+Backspace` | Delete last word |
| `Esc` | Back to menu (asks for confirmation once typing has started) |

**Result screen**

| Key | Action |
|-----|--------|
| `r` | Repeat the same words |
| `Enter` / `Tab` | New test with fresh words |
| `Esc` | Back to menu |

**Settings screen**

Rows: cursor shape, sound, volume, history expiry, difficulty.

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move between rows |
| `←` / `→` | Change the selected row's value |
| `0`-`9` / `Backspace` | Type an exact volume percentage (when on the volume row) |
| `Enter` | Save and return to menu |
| `Esc` | Return to menu (asks `y`/`n` to discard first if there are unsaved changes) |

**History screen**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Scroll |
| `Esc` / `q` | Back to menu |

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

Licensed under [GPL-3.0](LICENSE.md).

Word lists (`static/languages/`) and quote collections (`static/quotes/`) are taken from [monkeytypegame/monkeytype](https://github.com/monkeytypegame/monkeytype) and distributed under the same license. No modifications were made to the data files. The game logic, UI, and Rust source code are original work unaffiliated with the monkeytype project.
