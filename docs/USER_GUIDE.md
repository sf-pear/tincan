# Tincan User Guide

Tincan keeps a project's useful development memory in private Markdown. The
same workspace can contain no Git repository, one repository, or several nested
repositories.

## Memory

- **Plan:** current outcome-level work and ideas. It is a small living document.
- **Journal:** a dated snapshot of done, planned, open, and next.
- **Decision:** an accepted choice that constrains future work.
- **Learning:** a durable conclusion supported by evidence.
- **Global learning:** a user-approved learning available across projects.

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
After updating the CLI, run `tincan skill status` to check every detected
installation and `tincan skill install` when an update is available.
The interactive installer labels each destination as an install or update,
shows the complete Tincan skill path, and omits installations already current.

## Commands

| Command | Purpose |
| --- | --- |
| `init [PATH]` | Create private `.tincan/` storage. |
| `plan` | Print the living `.tincan/plan.md`; edit that file directly. |
| `journal` | Add concise bullets to today's historical work record. |
| `resume` | Print the living plan and latest journal when starting or wrapping up. |
| `decide STATEMENT` | Record an accepted choice. |
| `learn STATEMENT` | Record an evidence-supported reusable conclusion. |
| `search QUERY` | Search project records and global learnings. |
| `show UUID` | Print one complete project or global record. |
| `lift UUID --from FILE` | Save an approved generalized global learning. |
| `summary [PATH]` | Count stored memory; `--verbose` lists headings and paths. |
| `changes` | Match changed files across nested Git repos to records linked by `--file`. |
| `skill install` | Install the agent workflow into selected harnesses. |
| `skill status` | Check whether detected Agent Skill installations are current. |

Commands find the nearest parent `.tincan/config.toml`. Use `-d <path>` or
`--directory <path>` to start from another directory. Store `--file` values
relative to the Tincan workspace, such as `api/src/main.rs`.

## Journal

Every option is repeatable:

```powershell
tincan journal `
  --done "Implemented workspace discovery" `
  --planned "Add wrap-up guidance to the skill" `
  --question "Should plans later support priorities?" `
  --next "Exercise the workflow in a real multi-repo project"
```

Journal bullets are one-line summaries. Record accepted choices with `decide`
and durable evidence-backed conclusions with `learn`; do not duplicate them in
the journal. Their timestamps preserve when they were recorded. Keep the plan
limited to current direction, not implementation checklists or a duplicate of
an issue tracker.

Run `tincan --help` for complete syntax.

## Global learnings

Agents may propose lifting an evidence-supported project learning when it is
likely to change work in other projects. The agent first prepares and shows the
exact generalized Markdown body. Lifting always requires user approval; a
direct request such as "Make this a global learning" counts as approval.

```powershell
tincan lift 019c4ea8-7e42-7b31-a211-8df9357d747c --from global-learning.md
```

The prepared Markdown receives a new UUID and records the source workspace and
learning ID. Global learnings live under `~/.tincan/global/learnings/`; set
`TINCAN_HOME` to use a different personal Tincan directory. `search` and `show`
include them automatically and work outside a project when global storage is
available.

The input file contains Markdown body only: no YAML frontmatter, a non-empty H1
as its first content, and no second H1. Pass `--from -` to read the prepared
Markdown from stdin.

When work corrects or materially revises a project learning, search its UUID.
This finds global learnings whose `source_record` points to it. Review those
derived records and ask before updating them; corrections never propagate
automatically.
