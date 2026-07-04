---
paths:
  - "src/**/*.rs"
  - "static/languages/**/*.json"
  - "static/quotes/**/*.json"
---

# Architecture

**Data flow:** `main.rs` event loop → `App::on_key` / `App::tick` each frame → `ui::draw` renders

**Core types:**
- `Mode`: `Time(u64)` | `Words(usize)` | `Quote` — drives test configuration throughout
- `Difficulty`: `Normal` | `Expert` (any wrong char fails the test) | `Master` (wrong char in a completed word fails)
- `QuoteFilter`: `All` | `Short` | `Medium` | `Long` | `Thicc` — filters quote length when `Mode::Quote` is active

**Key invariant:** `GameState` is always rebuilt from scratch in `App::start_test()`; never mutate it incrementally between tests.

**Module roles:**

| Module | Role |
|---|---|
| `app.rs` | `App` struct: all UI state, input routing, screen transitions. Contains `word_lines()` used by both `app` and `ui` |
| `game.rs` | `GameState`: pure typing logic (WPM/accuracy/timers). No I/O |
| `ui/mod.rs` | Entry point (`draw()`) that dispatches to per-screen submodules based on `App::screen`. Holds the `Theme` palette (selected via `Settings::theme_idx`, applied per-frame through `th_*()` accessors) and layout helpers. Reads `App` + `GameState`, never mutates them |
| `ui/{menu,test_screen,result,history,help,settings}.rs` | One file per screen (`draw_menu`, `draw_test`, etc.), plus `help.rs` for the language picker overlay |
| `words.rs` | `LANGUAGES` static: word lists embedded at compile time via `include_str!`, cached per (lang, size). Also contains `load_quotes_for` |
| `pb.rs` | Personal best persistence (keyed by mode+lang) to `pb.json` |
| `sound.rs` | `SoundPack` (Off/Click/Pop) and audio output via `rodio` |
| `history.rs` | Persists results to `history.json` (max 50 entries) |
| `storage.rs` | Shared `data_dir()` (XDG/HOME/APPDATA) and atomic `write_atomic()` used by `history`/`pb` |
| `macros.rs` | `cycle_enum!` macro for settings enums that cycle via `next`/`prev`/`label` |

**Screen flow:** `Menu → Test → Result`, with `History`, `Help`, and `Settings` as overlays reachable from `Menu`. Overlays (language picker, quit/abandon confirm dialogs) are drawn on top in `ui::draw` regardless of the active screen.

**WPM formula:** correct chars ÷ 5 ÷ elapsed minutes (standard 5-chars-per-word). `raw_wpm` uses total chars typed.

**Adding a language:** Add word JSON to `static/languages/` → register a `lang!` entry in `words::LANGUAGES`. For quotes: add JSON to `static/quotes/` → add a `load_quotes_for` branch in `words.rs`.
