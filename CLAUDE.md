# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A terminal typing-speed game (monkeytype clone) built with `ratatui` + `crossterm`.
Provides word/quote typing tests, WPM/accuracy measurement, and result history persistence.

## Tech Stack

- **Language**: Rust (2024 edition)
- **TUI framework**: ratatui + crossterm
- **Serialization**: serde + serde_json
- **Build tool**: cargo
- **Audio**: rodio, `default-features = false` with only the `playback` feature enabled (requires `libasound-dev` on Linux / CoreAudio on macOS)

## Architecture, Code Style & Testing

Scoped rules load automatically when working with matching files — see `.claude/rules/`:

- `.claude/rules/architecture.md` — data flow, core types, module roles, screen flow, WPM formula, adding a language (`src/**/*.rs`, `static/languages,quotes/**/*.json`)
- `.claude/rules/code-style.md` — deliberate-simplification mode, `// NOTE:` markers, single-responsibility, `ui/` immutability (`src/**/*.rs`)
- `.claude/rules/testing.md` — `cargo test`, unit-test conventions (`src/**/*.rs`, `tests/**/*.rs`)

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

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/ko/v1.0.0/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- **type**: `feat` (new feature), `fix` (bug fix), `docs`, `style` (formatting, no logic change), `refactor`, `test`, `build`, `ci`, `chore` — others are allowed if they add clarity
- **description**: imperative mood, no trailing period (e.g. `add`, not `added`/`adds`)
- Breaking changes: append `!` after the type/scope (e.g. `feat!:`) and/or add a `BREAKING CHANGE:` footer
- Recent history in this repo already follows this convention (`docs:`, `build(deps):`, `ci:`, `style:`, `refactor:`) — match that style

## Before Committing

**Mandatory — no exceptions, no skipping:**

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

Run before every commit. Do not commit with clippy warnings, formatting issues, or failing tests. CI (`.github/workflows/ci.yml`) enforces the same checks on push/PR and will fail the build otherwise.

A `.claude/settings.json` `PostToolUse` hook also runs `rustfmt` and `cargo clippy -- -D warnings` automatically after Claude edits any `.rs` file, surfacing lint failures back to Claude mid-session.
