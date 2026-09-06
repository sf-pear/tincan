# Tincan User Guide

Tincan keeps a project's useful development memory in private Markdown. The
same workspace can contain no Git repository, one repository, or several nested
repositories.

## Memory

- **Plan:** a small living guide to current outcome-level direction.
- **Later:** an informal shelf of user-approved ideas and observations to revisit.
- **Journal:** a dated handoff with enough context to resume useful work.
- **Decision:** an accepted choice that constrains future work.
- **Learning:** a durable conclusion supported by evidence.
- **Global learning:** a user-approved learning available across projects.

The journal is not a complete project history. It preserves context that would
otherwise remain in someone's head: where work stopped, what remains unclear,
why the current state matters, and where work should resume. Do not duplicate
Git history, changelogs, issue trackers, or routine task summaries. The plan
reflects current direction rather than implementation steps.

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
| `remember TEXT` | Add one informal idea or observation to revisit later. |
| `later` | Print the informal `.tincan/later.md` shelf. |
| `journal` | Add concise context to today's project handoff. |
| `resume` | Print the plan, later shelf, and latest journal together. |
| `decide STATEMENT` | Record an accepted choice. |
| `learn STATEMENT` | Record an evidence-supported reusable conclusion. |
| `search QUERY` | Search project records and global learnings. |
| `show UUID` | Print one complete project or global record. |
| `lift UUID --from FILE` | Save an approved generalized global learning. |
| `review` | Show historical coverage and yearly record counts. |
| `review TIME_OPTION` | Print or save deterministic context for a retrospective. |
| `projects` | List registered workspaces and whether their paths are available. |
| `projects unregister PATH|UUID` | Forget one registration without changing project files. |
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

Journal bullets are concise handoff notes, not an inventory of completed work.
Before handing meaningful work back, review whether the next person has enough
context and update only what materially changed. Record accepted choices with
`decide` and durable evidence-backed conclusions with `learn`; do not duplicate
them in the journal. Review the plan at the same boundary, but change it only
when an outcome was completed, invalidated, refined, or revealed.

## Later

Use `tincan remember "..."` when an idea, observation, or minor issue is worth
revisiting but is not yet an intended outcome, accepted decision, or supported
learning. `tincan later` prints the shelf. Edit `.tincan/later.md` directly to
remove discarded items or move promoted items into the plan or a durable record.
Later items deliberately have no priority, status, assignment, or audit trail.
`resume` includes them so an agent can notice connections to the current work,
but an item is not current direction and must not be acted on merely because it
appears there.

Run `tincan --help` for complete syntax.

## Review

`review` keeps historical selection useful with or without an agent. With no
time option it shows the first and last recorded dates plus journal-day,
decision, and learning counts for each year. Explicit time options return readable Markdown
from the matching project records without calling a model or storing a derived
summary.

```powershell
tincan review
tincan review --year 2025
tincan review --month 2025-08
tincan review --quarter 2025-Q3
tincan review --half 2025-H2 --output review-2025-h2.md
tincan review --all-time
tincan review --year 2025 --all-projects
```

Time options are mutually exclusive and explicit. The current project is the
default; `--all-projects` reads records directly from every registered
workspace. `init` registers a stable workspace ID, and running any project
command after moving a workspace reconnects its path when the old location is
unavailable. If both locations still exist with the same ID, Tincan reports a
copy conflict instead of guessing; run `tincan init PATH` on the copy to assign
it a new ID. Unavailable or unreadable projects are skipped and reported.
Output prints to the terminal by default. `--output FILE` creates a Markdown file and
refuses to replace an existing file unless `--force` is also provided.

Use `tincan projects` to inspect the registry, then run `tincan projects
unregister PATH_OR_ID` to remove one entry. Unregistering never changes the
project's `.tincan/` directory. Later commands update the path of a registered
project but do not re-register one that was explicitly unregistered; rerun
`init` to register it. The registry contains stable IDs and absolute project
paths under `~/.tincan/projects/`, or `TINCAN_HOME/projects/` when configured;
it never contains copies of project records.

`review` replaces the former `summary` command. Use `search` and `show` when
looking for a specific record rather than a calendar period.

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
