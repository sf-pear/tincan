# Tincan

Capture the important decisions, the reasoning behind them, and record
learnings from agent coding sessions in plain Markdown, so anyone can pick up
where the work left off.

Tincan is deliberately not an agent runner, transcript database, vector store, or
publishing platform. Those can integrate later. Its job is to preserve the
engineering record that should survive changing agents and tools.

For concepts, daily workflows, collaboration, privacy, and command details, see
[the user guide](docs/USER_GUIDE.md).

## Install during development

```powershell
cargo install --path .
```

Install the bundled agent skill for Codex:

```powershell
tincan skill install
```

For another Agent Skills-compatible harness, pass its skills directory:

```powershell
tincan skill install --path C:\path\to\harness\skills
```

The skill is embedded in the executable, so installation does not require the
Tincan source checkout. The command writes outside `.tincan/` only to the selected
skills directory. It does not replace a differing installation unless
`--force` is supplied.

## Quick start

```powershell
tincan init C:\path\to\project
tincan inspect C:\path\to\project

tincan record attempt --repo C:\path\to\project `
  --title "Sequential gallery paging" `
  --summary "Test whether paging improves gallery responsiveness" `
  --result "Far scrolling required sequential catch-up requests" `
  --status failed `
  --affects app/page.tsx `
  --affects gallery-loading `
  --evidence "Release-build trace showed main-thread work was the bottleneck"

tincan record decision --repo C:\path\to\project `
  --title "Load full media details on demand" `
  --summary "Keep the gallery read model compact" `
  --result "Viewer resolves full detail only when opened" `
  --affects gallery-loading

tincan capture --repo C:\path\to\project --title "Gallery loading work"
tincan handoff --repo C:\path\to\project --title "Continue gallery refactor" `
  --next "Add stale-response regression test"
tincan field-note --repo C:\path\to\project `
  --from attempt-123-sequential-gallery-paging
tincan check --repo C:\path\to\project --changed
tincan search --repo C:\path\to\project "paging"
```

Initialization creates `.tincan/AGENT_GUIDE.md` and adds one instruction pointing
to it in the repository's root `AGENTS.md`. Existing instructions are
preserved, and repeating the command does not duplicate the Tincan instruction.

## Repository storage

```text
.tincan/
├── config.toml
├── attempts/
├── decisions/
├── learnings/
├── handoffs/
├── sessions/
└── field-notes/
```

Markdown files are canonical. Tincan reads them directly and derives search and
changed-file warnings without a database.

Apart from adding the Tincan guide pointer to `AGENTS.md` during initialization,
Tincan writes only within `.tincan/`. It never edits application source,
configuration, changelogs, or existing project documentation.

Teams may commit all or part of `.tincan/`. See the user guide for recommended
shared and private paths.

## Zed task

Add a task to your global Zed `tasks.json`:

```json
[
  {
    "label": "Tincan: Capture Current Work",
    "command": "tincan",
    "args": ["capture", "--repo", "$ZED_MAIN_GIT_WORKTREE"],
    "cwd": "$ZED_WORKTREE_ROOT",
    "use_new_terminal": true
  },
  {
    "label": "Tincan: Check Related History",
    "command": "tincan",
    "args": ["check", "--repo", "$ZED_MAIN_GIT_WORKTREE", "--changed"],
    "cwd": "$ZED_WORKTREE_ROOT",
    "use_new_terminal": true
  }
]
```
