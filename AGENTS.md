# Tincan Agent Guide

Tincan is a dependency-light Rust CLI. It writes portable Markdown into a
repository's `.tincan/` directory.

## Principles

- Markdown is canonical; indexes and richer analysis must be rebuildable.
- Raw transcripts are evidence, not accepted project truth.
- Attempts, failures, decisions, and superseded conclusions remain distinct.
- Keep integrations replaceable.
- Prefer deterministic path and topic matching before semantic infrastructure.
- Never change a target repository outside `.tincan/` unless a future command
  explicitly documents and confirms that behavior.

## Commands

```powershell
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo run -- --help
```

## Boundaries

- `cli.rs`: argument parsing and help text
- `commands.rs`: command orchestration and user output
- `git.rs`: read-only Git inspection
- `model.rs`: event types and portable Markdown rendering
- `store.rs`: `.tincan/` layout, writes, and scans
- `util.rs`: dependency-free formatting and path helpers

Do not add a database, background service, model SDK, graph engine, or
transcript-specific parser without evidence from real Tincan usage.

