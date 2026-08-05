# Tincan User Guide

Tincan preserves two kinds of durable development memory:

- A **decision** records what was chosen and why.
- A **learning** records what became known and the evidence supporting it.

It stores portable Markdown under `.tincan/`. There is no database, account,
background service, model provider, transcript importer, or publishing system.

## Install and initialize

```powershell
cargo install --path .
tincan skill install
tincan init C:\path\to\project
```

`tincan skill install --path C:\path\to\skills` installs the embedded agent
skill into another compatible harness. Add `--force` only when intentionally
replacing an older Tincan-owned installation.

Initialization creates:

```text
.tincan/
├── config.toml
├── decisions/
├── learnings/
└── AGENT_GUIDE.md
```

It also adds one pointer to the root `AGENTS.md`. Existing content is preserved,
and repeated initialization does not duplicate it.

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

Prefer a record that explains the reasoning:

> Paging introduced sequential requests without reducing measured renderer
> work. Keep the compact read model unless new traces show data loading has
> become the bottleneck.

Avoid routine progress updates, conversational speculation, raw transcripts,
credentials, private customer data, and conclusions unsupported by evidence.

Commit `.tincan/decisions/` and `.tincan/learnings/` when the repository's
development memory should be shared with collaborators.

## Scope

Tincan does not create session summaries, handoffs, field notes, blog posts,
release notes, commits, PRs, or publications. Separate tools may consume its
plain Markdown later without expanding Tincan's core responsibility.
