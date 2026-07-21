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
| `app/mod.rs` | `App` struct: all UI state, screen transitions, shared helpers (`start_test()`, `word_lines()` used by both `app` and `ui`). Declares the `app::{help,history,menu,result,settings,test}` submodules |
| `app/{menu,test,result,history,settings,help}.rs` | One file per screen holding that screen's `App::handle_*` input-routing methods (e.g. `handle_menu`, `handle_test`). `app/settings.rs` also defines `SettingsRow`, the single source of settings-row order shared with `ui/settings.rs` |
| `game.rs` | `GameState`: pure typing logic (WPM/accuracy/timers). No I/O |
| `ui/mod.rs` | Entry point (`draw()`) that dispatches to per-screen submodules based on `App::screen`, plus layout helpers. Reads `App` + `GameState`, never mutates them |
| `ui/theme.rs` | `Theme` palette (built-ins plus user themes loaded from `data_dir()/themes/*.json`, selected via `Settings::theme_name`) and the per-frame `th_*()` color accessors used by every `ui/*` screen |
| `ui/{menu,test_screen,result,history,help,settings}.rs` | One file per screen (`draw_menu`, `draw_test`, etc.), plus `help.rs` for the language picker overlay. Each screen's entry point only lays out regions and delegates one function per region (`test_screen`'s gauge/words/cursor/stats, `result`'s panels/chart) |
| `words.rs` | `LANGUAGES` static: word lists embedded at compile time via `include_str!`, cached per (lang, size). Also contains `load_quotes_for`, and `lang_at`/`lang_name` — the one place an out-of-range `lang_idx` falls back, so every screen names the same language |
| `pb.rs` | Personal best persistence (keyed by mode+lang) to `pb.json` |
| `sound.rs` | `SoundPack` (Off/Click/Pop) and audio output via `rodio` |
| `history.rs` | Persists results to `history.json` (max 50 entries) |
| `storage.rs` | Shared `data_dir()` (XDG/HOME/APPDATA), atomic `write_atomic()`, and the `load_json()`/`save_json()` pair every persisted file goes through (`config`/`history`/`pb`). Loads degrade to `Default` on a missing/malformed file |
| `macros.rs` | `cycle_enum!` macro for settings enums that cycle via `next`/`prev`/`label`, and exposes `ALL` — render the full set from that, never a hand-written second list |

**Screen flow:** `Menu → Test → Result`, with `History`, `Help`, and `Settings` as overlays reachable from `Menu`. Overlays (language picker, quit/abandon confirm dialogs) are drawn on top in `ui::draw` regardless of the active screen.

**WPM formula:** correct chars ÷ 5 ÷ elapsed minutes (standard 5-chars-per-word). `raw_wpm` uses total chars typed.

**Adding a language:** `lang!` derives every `include_str!` path from the name, so files must follow the convention: word lists at `static/languages/<lang>.json` (the implicit `default` size) and `static/languages/<lang>_<size>.json`, quotes at `static/quotes/<lang>.json`. Add the JSON, then register one line in `words::LANGUAGES` listing only the non-default sizes — `lang!("english", ["1k", "5k", "10k"], quotes)`. Omit the trailing `quotes` marker when there is no quote file. `load_quotes_for` resolves quotes from `LangDef.quotes`, so no code branch is needed.
