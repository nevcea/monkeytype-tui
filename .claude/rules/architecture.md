---
paths:
  - "src/**/*.rs"
  - "static/languages/**/*.json"
  - "static/quotes/**/*.json"
---

# Architecture

**Data flow:** `main.rs` event loop → `App::on_key` / `App::tick` each frame → `ui::draw` renders

**Core types:**
- `Mode`: `Time(u32)` | `Words(u32)` | `Quote` — drives test configuration throughout
- `Difficulty`: `Normal` | `Expert` (any wrong char resets test) | `Master` (wrong char before space resets)
- `QuoteFilter`: `All` | `Short` | `Medium` | `Long` | `Thicc` — filters quote length when `Mode::Quote` is active

**Key invariant:** `GameState` is always rebuilt from scratch in `App::start_test()`; never mutate it incrementally between tests.

**Module roles:**

| Module | Role |
|---|---|
| `app.rs` | `App` struct: all UI state, input routing, screen transitions. Contains `word_lines()` used by both `app` and `ui` |
| `game.rs` | `GameState`: pure typing logic (WPM/accuracy/timers). No I/O |
| `ui/mod.rs` | Entry point (`draw()`) that dispatches to per-screen submodules based on `App::screen`. Holds shared palette constants and layout helpers used across screens. Reads `App` + `GameState`, never mutates them |
| `ui/{menu,test_screen,result,history,help,settings}.rs` | One file per screen (`draw_menu`, `draw_test`, etc.), plus `help.rs` for the language picker overlay |
| `words.rs` | `LANGUAGES` static: word lists embedded at compile time via `include_str!`. Also contains `load_quotes_for` |
| `pb.rs` | Personal best persistence to `~/.local/share/monkeytype-tui/pb.json` |
| `sound.rs` | `SoundPack` (Off/Click/Pop) and audio output via `rodio` |
| `history.rs` | Persists results to `~/.local/share/monkeytype-tui/history.json` (max 50 entries) |

**Screen flow:** `Menu → Test → Result`, with `History`, `Help`, and `Settings` as overlays reachable from `Menu`. Overlays (language picker, quit/abandon confirm dialogs) are drawn on top in `ui::draw` regardless of the active screen.

**WPM formula:** correct chars ÷ 5 ÷ elapsed minutes (standard 5-chars-per-word). `raw_wpm` uses total chars typed.

**Adding a language:** Add word JSON to `static/languages/` → register a `lang!` entry in `words::LANGUAGES`. For quotes: add JSON to `static/quotes/` → add a `load_quotes_for` branch in `words.rs`.
