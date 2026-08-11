# Tincan

Tincan is a lightweight, plain-Markdown handoff between your past self, your
future self, and whichever coding agent works with you next.

When you return to a project, Tincan helps answer:

- Where did we stop?
- What is still open?
- What should happen next?
- What decisions must not be overlooked?

Agent-assisted development moves fast. Context gets lost between projects,
features, sessions, and agents. That wastes time and causes regressions when
important decisions are missed.

Tincan keeps a small record of what matters:

- **Plan:** the small, living list of intended outcomes.
- **Journal:** meaningful progress, open questions, plans, and next steps.
- **Decision:** an accepted choice that should guide future work.
- **Learning:** an evidence-supported conclusion worth remembering.

Tincan is designed to be maintained by the coding agent as work happens. With
the bundled skill installed, the agent records meaningful progress, accepted
decisions, useful learnings, open questions, and the next place to start. You
can run the same commands yourself whenever you want.

Nothing runs in the background or records everything automatically. The agent
uses Tincan deliberately to keep the small amount of context worth carrying
forward.

Everything is stored as plain Markdown inside the project. There is no database,
background service, graph, or attempt to record every action. Just a simple way
to stay sane given the sheer velocity of development and the amount of decisions
we make every day.

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

tincan plan
tincan decide "Load full media details on demand" --file web/app/page.tsx
tincan learn "Paging did not reduce rendering work" --evidence "Release trace"
tincan journal --done "Implemented the compact gallery read model" `
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
`-- journal/
```

When the workspace is inside Git, Tincan excludes `.tincan/` through Git's
local exclude file. A non-Git parent is already outside its nested repositories.
Markdown is canonical, and people or agents can edit record bodies and the plan
directly.

See [the user guide](docs/USER_GUIDE.md) or run `tincan --help`.
