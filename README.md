# Tincan

Capture important decisions, the reasoning behind them, and evidence-supported
learnings in plain Markdown so anyone can pick up where the work left off.

Tincan is a small, dependency-light Rust CLI. It is not an agent runner,
transcript database, publishing platform, or semantic search service. Markdown
under `.tincan/` is the source of truth.

See [the user guide](docs/USER_GUIDE.md) for the complete workflow.

## Install during development

```powershell
cargo install --path .
tincan skill install
```

Install the bundled skill into another Agent Skills-compatible harness:

```powershell
tincan skill install --path C:\path\to\harness\skills
```

Use `--force` to replace an older Tincan-owned skill installation.

## Quick start

```powershell
tincan init C:\path\to\project
tincan inspect C:\path\to\project

tincan record learning --repo C:\path\to\project `
  --title "Paging does not fix main-thread saturation" `
  --note "Paging added sequential catch-up requests without reducing the measured rendering bottleneck." `
  --file app/page.tsx `
  --topic gallery-loading `
  --evidence "Release-build trace"

tincan record decision --repo C:\path\to\project `
  --title "Load full media details on demand" `
  --note "Keep the gallery read model compact because rendering, not data loading, was the measured bottleneck." `
  --file app/page.tsx `
  --related learning-123-paging-does-not-fix-main-thread-saturation

tincan search --repo C:\path\to\project "gallery-loading"
tincan show --repo C:\path\to\project learning-123-paging-does-not-fix-main-thread-saturation
tincan check --repo C:\path\to\project --changed
```

Replace an active decision without erasing its reasoning:

```powershell
tincan record decision `
  --title "Preload adjacent media details" `
  --note "Preload only adjacent items after navigation measurements justified the bounded cost." `
  --supersedes decision-123-load-full-media-details-on-demand
```

## Repository storage

```text
.tincan/
├── config.toml
├── decisions/
└── learnings/
```

Initialization also creates `.tincan/AGENT_GUIDE.md` and adds one pointer to it
in the root `AGENTS.md`. Existing instructions are preserved and initialization
is idempotent.

Tincan reads record frontmatter for search and changed-file matching, then loads
the full Markdown only for `show`. Apart from the documented agent-guide pointer
and explicit skill installation, it never writes outside `.tincan/`.
