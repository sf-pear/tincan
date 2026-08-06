# Tincan

Tincan keeps important development memory in workspace-local Markdown. It helps
developers and agents preserve context without recording every action.

- **Plan:** the small, living list of intended outcomes.
- **Journal:** meaningful progress, decisions, learnings, open questions, and next steps.
- **Decision:** an accepted choice that should guide future work.
- **Learning:** an evidence-supported conclusion worth reusing.

## Install

Tincan requires Rust 1.85 or newer. Git is optional unless you use `changes`.

```powershell
cargo install tincan-cli
tincan skill install
```

From a source checkout, use `cargo install --path .`. After updating the CLI,
the next interactive Tincan command offers to update stale installed skills.

## Quick start

```powershell
tincan init C:\path\to\workspace
cd C:\path\to\workspace

tincan plan
tincan decide "Load full media details on demand" --file web/app/page.tsx
tincan learn "Paging did not reduce rendering work" --evidence "Release trace"
tincan journal --done "Implemented the compact gallery read model" `
  --decision "Load full media details on demand" `
  --planned "Add the stale-response regression test" `
  --next "Start with the stale-response test"

tincan resume
tincan search "gallery"
tincan summary
tincan changes
```

Commands find the nearest parent `.tincan/`. One workspace can therefore
contain zero, one, or several Git repositories. Pass `-d <path>` or
`--directory <path>` to start discovery from another directory.

## Storage

```text
.tincan/
|-- config.toml
|-- plan.md
|-- decisions/
|-- learnings/
|-- journal/
`-- AGENT_GUIDE.md
```

When the workspace is inside Git, Tincan excludes `.tincan/` through Git's
local exclude file. A non-Git parent is already outside its nested repositories.
Markdown is canonical, and people or agents can edit record bodies and the plan
directly.

See [the user guide](docs/USER_GUIDE.md) or run `tincan --help`.
