# Tincan

Tincan keeps important development memory in repository-local Markdown. It
helps developers and agents preserve context across sessions without recording
every action.

- **Journal:** meaningful progress, open questions, and next steps.
- **Decision:** an accepted choice that should guide future work.
- **Learning:** an evidence-supported conclusion worth reusing.

## Install

Tincan requires Git and Rust 1.85 or newer.

```powershell
cargo install tincan-cli
tincan skill install
```

The skill installer detects supported user-wide agent harnesses and asks where
to install the bundled skill. Use `--path <skills-directory>` for another
location and `--force` to update an existing installation.

From a source checkout, use `cargo install --path .`.

After updating the CLI, the next interactive Tincan command offers to update
installed skills that differ from the newly bundled version.

## Quick start

```powershell
tincan init C:\path\to\project
cd C:\path\to\project

tincan journal `
  --done "Implemented the compact gallery read model" `
  --question "Should adjacent details be preloaded?" `
  --next "Add the stale-response regression test"

tincan decide "Load full media details on demand" --file app/page.tsx
tincan learn "Paging did not reduce rendering work" `
  --file app/page.tsx `
  --evidence "Release-build trace"

tincan resume
tincan search "gallery"
tincan show 019c4ea8-7e42-7b31-a211-8df9357d747c
tincan summary
tincan changes
```

Commands use the current directory by default. Pass `-d <path>` or
`--directory <path>` to target another directory.

## Storage

```text
.tincan/
├── config.toml
├── decisions/
├── learnings/
├── journal/
└── AGENT_GUIDE.md
```

Tincan keeps `.tincan/` out of normal Git tracking through the repository's
local exclude file. Markdown is canonical. `decide` and `learn` create
UUID-named files with validated frontmatter; developers and agents can add
detail directly to their Markdown bodies.

See [the user guide](docs/USER_GUIDE.md) or run `tincan --help` for command
details.
