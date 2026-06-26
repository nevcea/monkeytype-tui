# monkeytype-tui

A terminal typing-speed game inspired by [monkeytype](https://monkeytype.com), built with Rust + ratatui.

![Rust](https://img.shields.io/badge/rust-2024_edition-orange)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

## Features

- **Three modes** — Time (5s–custom), Words (10–500, custom), Quote
- **Punctuation & numbers** toggles
- **25+ languages** with word lists up to 10k entries, embedded at compile time
- **Quotes** in 22 languages
- **WPM / raw WPM / accuracy** on the result screen (standard 5-chars-per-word formula)
- **Persistent history** — last 50 results saved to `~/.local/share/monkeytype-tui/history.json`
- **Language picker** with fuzzy search

## Install

```bash
cargo install --path .
```

Or just run directly:

```bash
cargo run --release
```

Requires a terminal that supports 256 colors and Unicode.

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Restart / back to menu |
| `Esc` | Back / cancel |
| `Ctrl+C` / `q` | Quit (from menu) |
| `↑` / `↓` | Navigate menus |
| `←` / `→` | Switch mode tabs |
| `p` | Toggle punctuation |
| `n` | Toggle numbers |
| `l` | Open language picker |
| `h` | Open history |
| `?` | Open help |
| `Enter` | Confirm / start test |

## Supported Languages

Albanian, Bosnian, Catalan, Czech, Danish, Dutch, English, Finnish, French, German, Greek, Hungarian, Italian, Japanese, Korean, Latin, Norwegian, Polish, Portuguese, Romanian, Russian, Spanish, Swedish, Turkish, Ukrainian, Vietnamese

## Development

```bash
cargo build       # compile
cargo run         # run
cargo test        # run tests
cargo clippy      # lint
```

### Adding a language

1. Add the word list JSON to `static/languages/`
2. Add a `lang!` entry in `src/words.rs`
3. If quotes exist, add a branch in `src/quotes.rs`

## Architecture

```
main.rs       event loop (50ms tick)
app.rs        App struct — UI state, input routing, screen transitions
game.rs       GameState — pure typing logic, WPM/accuracy, timers
ui.rs         all rendering (reads App+GameState, never mutates)
words.rs      LANGUAGES static, word lists embedded via include_str!
quotes.rs     quote JSON loader
history.rs    persistence (~/.local/share/monkeytype-tui/history.json)
```

## License

GPL-3.0
