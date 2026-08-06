# Tincan User Guide

Tincan keeps a project's useful development memory in private Markdown. The
same workspace can contain no Git repository, one repository, or several nested
repositories.

## Memory

- **Plan:** current outcome-level work and ideas. It is a small living document.
- **Journal:** a dated snapshot of done, decided, learned, planned, open, and next.
- **Decision:** an accepted choice that constrains future work.
- **Learning:** a durable conclusion supported by evidence.

The journal preserves meaningful project history and handoff context. Transient
status belongs there only when it explains progress, a blocker, an open
question, or where work should resume. The plan reflects the current direction.

A learning must remain useful beyond the current session and change a future
implementation, debugging, review, or architectural action. It should prevent a
concrete mistake, regression, or repeated investigation. Tool usage, memory
maintenance, routine verification with no durable implication, and transient
dependency or audit status are not learnings.

## Setup

```powershell
cargo install tincan-cli
tincan skill install
tincan init C:\path\to\workspace
```

`init` creates `.tincan/`. When the workspace is inside a Git repository,
Tincan verifies that the folder is untracked and adds it to that repository's
local exclude file. A non-Git parent needs no ignore rule for its child repos.

## Commands

| Command | Purpose |
| --- | --- |
| `init [PATH]` | Create private `.tincan/` storage. |
| `plan` | Print the living `.tincan/plan.md`; edit that file directly. |
| `journal` | Add concise bullets to today's historical work record. |
| `resume` | Print the latest journal when starting or wrapping up. |
| `decide STATEMENT` | Record an accepted choice. |
| `learn STATEMENT` | Record an evidence-supported reusable conclusion. |
| `search QUERY` | Search record headings, metadata, and bodies. |
| `show UUID` | Print one complete decision or learning. |
| `summary [PATH]` | Count stored memory; `--verbose` lists headings and paths. |
| `changes` | Match changed files across nested Git repos to records linked by `--file`. |
| `skill install` | Install the agent workflow into selected harnesses. |

Commands find the nearest parent `.tincan/config.toml`. Use `-d <path>` or
`--directory <path>` to start from another directory. Store `--file` values
relative to the Tincan workspace, such as `api/src/main.rs`.

## Journal

Every option is repeatable:

```powershell
tincan journal `
  --done "Implemented workspace discovery" `
  --decision "One Tincan workspace may contain several repositories" `
  --learning "Persistent plan.md is not a cross-agent convention" `
  --planned "Add wrap-up guidance to the skill" `
  --question "Should plans later support priorities?" `
  --next "Exercise the workflow in a real multi-repo project"
```

Journal bullets are one-line summaries. Put lasting reasoning in decision or
learning records. Keep the plan limited to current direction, not implementation
checklists or a duplicate of an issue tracker.

Run `tincan --help` for complete syntax.
