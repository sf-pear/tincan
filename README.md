# Tincan

Journal meaningful development progress, preserve important decisions and
evidence-supported learnings, and pick up where the work left off.

Tincan is a small, dependency-light Rust CLI. It is not an agent runner,
transcript database, publishing platform, or semantic search service. Markdown
under `.tincan/` is the source of truth.

See [the user guide](docs/USER_GUIDE.md) for the complete workflow.

## Install

Tincan requires Git and Rust 1.85 or newer.

```powershell
cargo install tincan-cli
tincan --version
tincan skill install
```

When working from a source checkout, use `cargo install --path .` instead.

Install the bundled skill into another Agent Skills-compatible harness:

```powershell
tincan skill install --path C:\path\to\harness\skills
```

Use `--force` to replace an older Tincan-owned skill installation.

## Quick start

```powershell
tincan init C:\path\to\project
tincan inspect C:\path\to\project

tincan journal --repo C:\path\to\project `
  --done "Implemented the compact gallery read model" `
  --question "Should adjacent details be preloaded?" `
  --next "Add the stale-response regression test"

tincan resume --repo C:\path\to\project

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
├── learnings/
├── journal/
└── AGENT_GUIDE.md
```

Initialization adds `/.tincan/` to Git's local exclude file and verifies the
rule with Git. Tincan refuses to initialize if `.tincan/` already contains
tracked files. Memory stays inside the checkout and is omitted from normal Git
status, add, and push workflows. An explicit force-add can override any Git
ignore rule, so do not force-add private Tincan files. Initialization does not
modify the repository's tracked `AGENTS.md` or `.gitignore`.

Tincan reads record frontmatter for search and changed-file matching, then loads
the full Markdown only for `show`. Explicit skill installation is the only
operation that writes outside the repository and its local Git metadata.
