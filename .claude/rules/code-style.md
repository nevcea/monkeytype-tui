---
paths:
  - "src/**/*.rs"
---

# Code Style

- **Ponytail mode** is the default: YAGNI, smallest working diff, no speculative abstractions
- Mark deliberate simplifications with a `// ponytail: <reason>` comment
- Single responsibility per function; `ui/` must never mutate state
- Logic shared between `app` and `ui` lives in `app/mod.rs` (e.g. `word_lines()`); each screen's input handling lives in its own `app/<screen>.rs`
