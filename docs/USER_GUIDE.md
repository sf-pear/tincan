# Tincan User Guide

Tincan preserves three complementary kinds of development memory:

- A **journal** records meaningful work completed today, open questions, and
  concrete next steps.
- A **decision** records what was chosen and why.
- A **learning** records what became known and the evidence supporting it.

It stores portable Markdown under `.tincan/`. There is no database, account,
background service, model provider, transcript importer, or publishing system.

## Install and initialize

```powershell
cargo install tincan-cli
tincan --version
tincan skill install
tincan init C:\path\to\project
```

Installation requires Git and Rust 1.85 or newer. From a source checkout, use
`cargo install --path .` instead.

`tincan skill install --path C:\path\to\skills` installs the embedded agent
skill into another compatible harness. Add `--force` only when intentionally
replacing an older Tincan-owned installation.

Initialization creates:

```text
.tincan/
├── config.toml
├── decisions/
├── learnings/
├── journal/
└── AGENT_GUIDE.md
```

Tincan adds `/.tincan/` to `.git/info/exclude`, which is local to the checkout,
and asks Git to verify that `.tincan/config.toml` is ignored. It refuses to
initialize when `.tincan/` already contains tracked files. The memory remains
available to local agents without appearing in normal `git status`, `git add`,
or push workflows. An explicit force-add can override any Git ignore rule, so
do not force-add private Tincan files. Initialization does not modify tracked
`AGENTS.md` or `.gitignore` files and is safe to repeat.

## Keep a daily journal

The journal is one living Markdown file per local calendar day:

```text
.tincan/journal/2026-08-05.md
```

Add concise bullets as meaningful work develops:

```powershell
tincan journal `
  --done "Implemented the compact gallery read model" `
  --question "Should adjacent details be preloaded?" `
  --next "Add the stale-response regression test"
```

Each option is repeatable, and at least one is required. Exact duplicates are
ignored. When a `--done` bullet exactly matches an existing `Next` bullet,
Tincan moves it to `Done`.

Return to the latest journal entry with:

```powershell
tincan resume
```

Use the journal for chronological working state, not extensive reasoning.

## Record a learning

Use a learning when evidence supports a conclusion worth reusing. Failed
approaches belong here when they taught the project something durable.

```powershell
tincan record learning `
  --title "Paging does not fix main-thread saturation" `
  --note "Paging added sequential catch-up requests without reducing the measured rendering bottleneck." `
  --file src/gallery.rs `
  --topic performance `
  --evidence "Release-build trace"
```

Learnings have no status. The note should explain what became known, why it is
believed, where it applies, and when it may need reconsideration.

## Record a decision

```powershell
tincan record decision `
  --title "Load full media details on demand" `
  --note "Keep the read model compact because traces identified rendering as the bottleneck." `
  --file src/gallery.rs `
  --related learning-123-paging-does-not-fix-main-thread-saturation
```

Decisions are created as `active`; callers cannot choose or invent their status.
When a later decision replaces an active one, use:

```powershell
tincan record decision `
  --title "Preload adjacent media details" `
  --note "Preload only adjacent items because measurements now justify the bounded cost." `
  --supersedes decision-123-load-full-media-details-on-demand
```

Tincan marks the earlier decision `superseded`, adds `superseded_by` to it, and
adds `supersedes` to the replacement. The earlier note and evidence remain.

## Metadata

Both record types accept:

- `--file` for repeatable repository-relative paths;
- `--topic` for repeatable discovery terms;
- `--evidence` for repeatable supporting observations;
- `--related` for repeatable record IDs.

`--supersedes` is available only for decisions. Record commands validate every
option before writing. Unknown options, duplicate single-value options, and
invalid relationships return corrective errors so callers can fix and retry.

## Find relevant memory

```powershell
tincan search "gallery-loading"
tincan show learning-123-paging-does-not-fix-main-thread-saturation
tincan check --changed
tincan inspect
```

Search matches compact frontmatter such as IDs, titles, types, decision status,
files, topics, and relationships. `show` loads the selected full Markdown file.
`check --changed` deterministically matches Git-changed paths against record
`files`; it does not use embeddings or semantic guesses.

Tincan validates manually edited frontmatter while scanning. Decisions must be
`active` or `superseded`, and learnings must not contain a status.

## What to preserve

Classify information by the question it answers:

| Type | Question | Examples |
| --- | --- | --- |
| Journal | What happened today, what is open, and what comes next? | Implemented feature, unresolved question, unfinished task |
| Learning | What evidence-supported knowledge will remain useful beyond today? | Measured limitation, failure that revealed a durable constraint |
| Decision | What accepted choice now constrains future work, and why? | Architecture, convention, replacement of an earlier choice |

Do not turn an open question, todo, progress update, or proposal into a learning
or decision. A completed change may have one short journal bullet and a durable
record, but avoid repeating the reasoning: the journal says what progressed;
the learning or decision explains why it matters.

Prefer a record that explains the reasoning:

> Paging introduced sequential requests without reducing measured renderer
> work. Keep the compact read model unless new traces show data loading has
> become the bottleneck.

Avoid routine progress updates, conversational speculation, raw transcripts,
credentials, private customer data, and conclusions unsupported by evidence.

Tincan memory is private by default. A future repository setting may make the
folder deliberately committable; until that mode exists, do not force-add
private `.tincan/` files merely to share them casually.

## Scope

Tincan does not create blog posts, release notes, commits, PRs, or publications.
Separate tools may consume its plain Markdown later without expanding Tincan's
core responsibility.
