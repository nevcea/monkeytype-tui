# monkeytype-tui

> A monkeytype-style typing speed game that runs entirely in your terminal.

[![CI](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/nevcea/monkeytype-tui/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-2024_edition-orange)
![License](https://img.shields.io/badge/license-GPL--3.0-blue)

## Highlights

- **No browser, no Electron** — runs entirely in your terminal, built with
  [ratatui](https://github.com/ratatui-org/ratatui) and
  [crossterm](https://github.com/crossterm-rs/crossterm)
- **48 languages**, word lists and quotes embedded at compile time — no
  network access, no runtime dependencies beyond the binary
- **Strict difficulty modes** — Expert fails on any mistake, Master fails if
  the word you just finished had one, on top of a 75% accuracy floor for
  every run
- **26 built-in themes** plus support for custom themes dropped into your
  data directory

## Installation

**Requires:** Rust + Cargo ([install](https://rustup.rs)), a terminal with
256-color and Unicode support, and `libasound-dev` on Linux (for audio).

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run --release
```

## Usage

Launch the app, use `1`/`2`/`3` to pick Time / Words / Quote, `←`/`→` to change
the option, then press `Enter`/`Tab` to start typing.

| Key | Action |
| ----- | -------- |
| `1` / `2` / `3` | Switch to Time / Words / Quote mode |
| `←` / `→` | Change mode option |
| `Enter` / `Tab` | Start test |
| `l` | Open language picker |
| `t` | Open theme picker |
| `s` | Open settings |
| `h` | Open history |
| `?` | Open help |
| `q` / `Esc` / `Ctrl+C` | Quit |

While typing, `Backspace` deletes a character, `Ctrl+Backspace` deletes a
word, and `Esc` returns to the menu.

Every screen has its own controls shown contextually — press `?` at any time
for the full in-app reference.

## Contributing

Bug reports, feature requests, and pull requests are welcome — open an issue to
start the conversation.

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

This project is licensed under the **[GNU GPL-3.0-only](LICENSE.md)**. In
short: you're free to use, modify, and redistribute this software, but any
distributed derivative work must also be licensed under GPL-3.0 and its
source made available.

**Third-party data:** word lists (`static/languages/`) and quote collections
(`static/quotes/`) are copied unmodified from
[monkeytypegame/monkeytype](https://github.com/monkeytypegame/monkeytype),
which is also GPL-3.0-licensed. All game logic, UI, and Rust source code in
this repository are original work and unaffiliated with the monkeytype
project.
