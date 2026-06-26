# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # compile
cargo run            # run the TUI app
cargo test           # run all tests
cargo test <name>    # run a single test by name substring
cargo clippy         # lint
```

## Architecture

A terminal typing-speed game (monkeytype clone) built with `ratatui` + `crossterm`.

**Data flow:** `main.rs` owns the event loop → calls `App::on_key` and `App::tick` each frame → `ui::draw` renders from `App` state.

**Module roles:**
- `app.rs` — `App` struct: all UI state (current screen, menu selections, scroll), input routing, and screen transitions. Also contains `word_lines()` for the word-wrap layout used by both `app` and `ui`.
- `game.rs` — `GameState`: pure typing logic. Tracks chars typed, cursor position, WPM/accuracy, timers. No I/O.
- `ui.rs` — all rendering. Reads from `App` + `GameState`, never mutates them.
- `words.rs` — `LANGUAGES` static: word lists embedded at compile time via `include_str!`. `load_words(lang_idx, size_idx)` returns the word vec.
- `quotes.rs` — loads quote JSON for a language name; called on language change.
- `history.rs` — persists results to `~/.local/share/monkeytype-tui/history.json` (max 50 entries).

**Screens:** `Menu → Test → Result`, plus `History` and `Help` as overlays reachable from `Menu`. `App::screen` drives which handler fires in `on_key`.

**Word list embedding:** All language JSON files in `static/languages/` are embedded at compile time. Adding a new language requires a `lang!` entry in `words::LANGUAGES` and a corresponding `load_quotes_for` branch in `quotes.rs` if quotes exist.

**WPM formula:** correct chars ÷ 5 ÷ elapsed minutes (standard 5-chars-per-word). `raw_wpm` uses total chars typed instead of correct chars.
