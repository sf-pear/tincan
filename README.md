# Tincan

Tincan is a lightweight, plain-Markdown handoff between your past self, your
future self, and whichever coding agent works with you next.

When you return to a project, Tincan helps answer:

- Where did we stop?
- What is still open?
- What should happen next?
- What decisions must not be overlooked?
- What did another project already teach us?

Agent-assisted development moves fast. Context gets lost between projects,
features, sessions, and agents. That wastes time and causes regressions when
important decisions are missed.

Tincan keeps a small record of what matters:

- **Plan:** the small, living list of intended outcomes.
- **Journal:** meaningful progress, open questions, plans, and next steps.
- **Decision:** an accepted choice that should guide future work.
- **Learning:** an evidence-supported conclusion worth remembering.
- **Global learning:** a user-approved learning available across projects.

Tincan is designed to be maintained by the coding agent as work happens. With
the bundled skill installed, the agent records meaningful progress, accepted
decisions, useful learnings, open questions, and the next place to start. You
can run the same commands yourself whenever you want.

Nothing runs in the background or records everything automatically. The agent
uses Tincan deliberately to keep the small amount of context worth carrying
forward.

Everything is stored as plain Markdown. Project memory stays inside the project,
while approved global learnings live under `~/.tincan/global/learnings/`. There
is no database, background service, graph, or attempt to record every action.
Just a simple way to stay sane given the sheer velocity of development and the
amount of decisions we make every day.

## Install

Tincan requires Rust 1.85 or newer. Git is optional unless you use `changes`.

```powershell
cargo install tincan-cli
tincan skill install
```

From a source checkout, use `cargo install --path . --force`. After updating
the CLI, the next successful interactive Tincan command reports stale installed
skills and directs you to run `tincan skill install`. Use
`tincan skill status` for a concise read-only health check that also works in
agent-driven and non-interactive terminals; it groups installations that need
updates separately from those already current.

## Quick start

```powershell
tincan init C:\path\to\workspace
cd C:\path\to\workspace

tincan resume
tincan decide "Load full media details on demand" --file web/app/page.tsx
tincan learn "Paging did not reduce rendering work" --evidence "Release trace"
tincan journal --done "Implemented the compact gallery read model" `
  --planned "Add the stale-response regression test" `
  --next "Start with the stale-response test"

tincan search "gallery"
tincan summary
tincan changes
```

Commands find the nearest parent `.tincan/`. One workspace can therefore
contain zero, one, or several Git repositories. Pass `-d <path>` or
`--directory <path>` to start discovery from another directory.

## Storage

Project memory:

```text
.tincan/
|-- config.toml
|-- plan.md
|-- decisions/
|-- learnings/
`-- journal/
```

User-approved global learnings:

```text
~/.tincan/
`-- global/
    `-- learnings/
```

When the workspace is inside Git, Tincan excludes `.tincan/` through Git's
local exclude file. A non-Git parent is already outside its nested repositories.
Markdown is canonical, and people or agents can edit record bodies and the plan
directly. After an agent shows a generalized Markdown draft and receives
approval, `tincan lift UUID --from FILE` writes it into
`~/.tincan/global/learnings/` with a new ID and source provenance. `search` and
`show` include global learnings automatically, even when run outside a project.
Searching a project learning's UUID also finds global learnings derived from it.
Set `TINCAN_HOME` to relocate the personal `.tincan` directory.

## Global learnings

When a project learning is likely to help elsewhere, the agent prepares and
shows a standalone generalized Markdown draft. After approval, save that draft
without YAML frontmatter and lift it:

```powershell
tincan lift <project-learning-id> --from global-learning.md
```

The draft must begin with exactly one non-empty H1. Use `--from -` to read it
from stdin instead of a file. Tincan gives the global learning its own UUID and
records the project learning in `source_record`; the detailed project record is
left unchanged.

If later work corrects a project learning, searching its UUID finds global
learnings derived from it so the agent can ask whether they also need revision.
Corrections never propagate automatically.

See [the user guide](docs/USER_GUIDE.md) or run `tincan --help`.
