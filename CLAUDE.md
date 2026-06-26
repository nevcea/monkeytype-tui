# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Project Overview

A terminal typing-speed game (monkeytype clone) built with `ratatui` + `crossterm`.
Provides word/quote typing tests, WPM/accuracy measurement, and result history persistence.

## Tech Stack

- **Language**: Rust (2024 edition)
- **TUI framework**: ratatui + crossterm
- **Serialization**: serde + serde_json
- **Build tool**: cargo
- **Audio**: rodio (requires `libasound-dev` on Linux / CoreAudio on macOS)

## Architecture

**Data flow:** `main.rs` event loop → `App::on_key` / `App::tick` each frame → `ui::draw` renders

**Core types:**
- `Mode`: `Time(u32)` | `Words(u32)` | `Quote` — drives test configuration throughout
- `Difficulty`: `Normal` | `Expert` (any wrong char resets test) | `Master` (wrong char before space resets)

**Key invariant:** `GameState` is always rebuilt from scratch in `App::start_test()`; never mutate it incrementally between tests.

**Module roles:**

| Module | Role |
|---|---|
| `app.rs` | `App` struct: all UI state, input routing, screen transitions. Contains `word_lines()` used by both `app` and `ui` |
| `game.rs` | `GameState`: pure typing logic (WPM/accuracy/timers). No I/O |
| `ui.rs` | All rendering. Reads `App` + `GameState`, never mutates them |
| `words.rs` | `LANGUAGES` static: word lists embedded at compile time via `include_str!`. Also contains `load_quotes_for` |
| `pb.rs` | Personal best persistence to `~/.local/share/monkeytype-tui/pb.json` |
| `sound.rs` | `SoundPack` (Off/Click/Pop) and audio output via `rodio` |
| `history.rs` | Persists results to `~/.local/share/monkeytype-tui/history.json` (max 50 entries) |

**Screen flow:** `Menu → Test → Result`, with `History`, `Help`, and `Settings` as overlays reachable from `Menu`.

**WPM formula:** correct chars ÷ 5 ÷ elapsed minutes (standard 5-chars-per-word). `raw_wpm` uses total chars typed.

**Adding a language:** Add word JSON to `static/languages/` → register a `lang!` entry in `words::LANGUAGES`. For quotes: add JSON to `static/quotes/` → add a `load_quotes_for` branch in `words.rs`.

## Code Style

- **Ponytail mode** is the default: YAGNI, smallest working diff, no speculative abstractions
- Mark deliberate simplifications with a `// ponytail: <reason>` comment
- Single responsibility per function; `ui.rs` must never mutate state
- Logic shared between `app` and `ui` lives in `app.rs` (e.g. `word_lines()`)

## Testing

- Run all tests with `cargo test`
- Unit tests target pure functions in `game.rs`
- For non-trivial logic, leave the smallest failing check that detects a regression

## Commands

```bash
cargo build              # compile
cargo run                # run the TUI app
cargo test               # run all tests
cargo test <name>        # run a single test by name substring
cargo clippy             # lint
cargo fmt                # format code
cargo fmt -- --check     # check formatting (required by CI)
```

## Before Committing

**Mandatory — no exceptions, no skipping:**

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

Run before every commit. Do not commit with clippy warnings, formatting issues, or failing tests.
