# Tincan User Guide

Capture the important decisions, the reasoning behind them, and record
learnings from agent coding sessions in plain Markdown, so anyone can pick up
where the work left off.

It records:

- what was attempted;
- why an attempt succeeded or failed;
- evidence supporting the conclusion;
- accepted decisions;
- reusable learnings;
- the state another person or agent needs to continue;
- possible field notes.

Tincan stores everything as Markdown under `.tincan/` in the selected Git repository.
There is no database, account, background service, or required AI provider.

## Safety boundary

Tincan writes inside the repository's `.tincan/` directory. During `tincan init`, it
also adds one Tincan guide pointer to the root `AGENTS.md`.

It does not modify:

- application source code;
- package or build configuration;
- `CHANGELOG.md`;
- existing documentation;
- Git branches, commits, tags, or remotes.

Future integrations that publish, update changelogs, or write outside `.tincan/`
must be separate commands with explicit confirmation.

## Install

From the Tincan source repository:

```powershell
cargo install --path .
```

Confirm that the executable is available:

```powershell
tincan --help
```

## Install the agent skill

Tincan includes a small skill that teaches compatible agents when to search
history and when an attempt, decision, learning, session, or handoff is worth
preserving.

Install it into the default Codex user skills directory:

```powershell
tincan skill install
```

`CODEX_HOME` is honored when set. Otherwise Tincan uses `.codex/skills` under the
current user's home directory.

Install it into another harness's skills directory with:

```powershell
tincan skill install --path C:\path\to\harness\skills
```

The destination will contain `tincan/SKILL.md` and Codex UI metadata. The skill is
embedded in the Tincan executable and does not require a source checkout.

Installation is idempotent. If an existing `tincan` skill differs from the version
bundled with the executable, Tincan stops rather than overwriting it. Review the
destination and use `--force` to update Tincan-owned files intentionally.

This command is an explicit exception to Tincan's repository write boundary: it
writes only to the selected harness skills directory. It does not initialize a
repository or change project files.

During Tincan development, commands can also be run without installation:

```powershell
cargo run -- --help
```

## Initialize a project

Run:

```powershell
tincan init C:\path\to\project
```

From inside the project, the path can be omitted:

```powershell
tincan init
```

Tincan creates:

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

Tincan also creates `.tincan/AGENT_GUIDE.md`, which contains a reusable operating
contract for development agents. It creates or appends one line to the root
`AGENTS.md` so agents can discover that guide. Existing content is preserved,
and repeated runs do not duplicate the instruction.

Running `tincan init` again is safe. Commands also add missing directories when a
new Tincan version introduces another artifact type.

## Decide what to commit

For an individual project, committing all reviewed `.tincan/` records is the
simplest choice.

For collaborative projects, these are useful to share:

```text
.tincan/attempts/
.tincan/decisions/
.tincan/learnings/
.tincan/handoffs/
```

Session and field-note drafts may contain incomplete reasoning or private
details. To keep those local, add `.tincan/.gitignore`:

```gitignore
/sessions/
/field-notes/
```

Do not record credentials, access tokens, private customer data, or complete raw
transcripts in committed records.

## Daily workflow

### 1. Check relevant history

Before or during work:

```powershell
tincan check --changed
```

Tincan compares changed and untracked files with the `affects` entries in previous
records. If a file overlaps an earlier failed attempt or decision, Tincan shows the
related record.

This is deterministic path matching. It does not use embeddings or guess at
semantic relationships.

### 2. Record an attempt

Record an experiment when its outcome will affect future work:

```powershell
tincan record attempt `
  --title "Sequential gallery paging" `
  --summary "Test whether paging improves gallery responsiveness" `
  --result "Far scrolling required sequential catch-up requests" `
  --status failed `
  --affects src/gallery.rs `
  --affects gallery-loading `
  --topic performance `
  --evidence "Release-build trace showed main-thread work was the bottleneck"
```

Useful attempt statuses include:

- `proposed`
- `in-progress`
- `succeeded`
- `failed`
- `inconclusive`
- `superseded`

After creation, open the Markdown file and complete its conclusion:

> What should a future person or agent do—or avoid—because of this result?

### 3. Record a decision

```powershell
tincan record decision `
  --title "Load full media details on demand" `
  --summary "Keep the gallery read model compact" `
  --result "Resolve full details only when the viewer opens" `
  --affects gallery-loading `
  --related attempt-123-sequential-gallery-paging
```

Use `--related` to connect the decision to attempts or learnings that justify
it. The value should be another record's `id`.

### 4. Record a learning

```powershell
tincan record learning `
  --title "Paging does not fix main-thread saturation" `
  --summary "Measure rendering work before introducing paged data loading" `
  --affects gallery-loading `
  --topic performance `
  --evidence "Compact read model met the interaction budget"
```

A learning should be reusable and supported by evidence. Routine summaries
belong in session records instead.

### 5. Capture the session

```powershell
tincan capture --title "Gallery loading work"
```

Tincan creates a draft containing:

- repository and branch;
- working-tree status;
- affected files;
- diff statistics;
- recent commits;
- sections for outcomes, decisions, attempts, verification, and learnings.

Tincan does not claim to understand the session automatically. Review the file and
replace the prompts with accepted conclusions.

### 6. Create a handoff

```powershell
tincan handoff `
  --title "Continue gallery refactor" `
  --next "Add a stale-response regression test" `
  --next "Run the far-scroll release-build scenario"
```

A handoff should let a fresh person or agent continue without reading the
original conversation. Complete:

- what is now true;
- what is finished;
- exact next actions;
- accepted decisions;
- risks and verification gaps;
- uncommitted working-tree state.

### 7. Draft a field note

Find the source record ID:

```powershell
tincan search "paging"
```

Then create a draft:

```powershell
tincan field-note `
  --from attempt-123-sequential-gallery-paging `
  --title "Why sequential paging made our gallery worse"
```

The field-note template asks:

- What did I expect?
- What happened?
- What evidence changed the decision?
- What did I change?
- What can another project reuse?

Field notes are drafts. Review private details before publishing.

## Search

Search titles, frontmatter, and Markdown bodies:

```powershell
tincan search "SQLite migration"
tincan search "gallery-loading"
tincan search "failed"
```

Search is currently case-insensitive text matching. Record IDs and topics appear
in results.

## Inspect a project

```powershell
tincan inspect
```

This reports:

- canonical repository root;
- current branch;
- number of changed files;
- total Tincan records;
- counts by artifact type.

## Working with multiple agents

Every agent should:

1. read `AGENTS.md`;
2. run `tincan check --changed` before modifying a sensitive area;
3. search Tincan when a task resembles previous work;
4. record important failed attempts and accepted decisions;
5. create a handoff before switching agents;
6. avoid treating raw transcript statements as accepted truth.

Tincan records are agent-neutral Markdown. Codex, Claude, Pi, Zed Agent, and human
collaborators read the same material.

## Zed integration

Add project-aware tasks to the global Zed `tasks.json`:

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

The tasks appear in Zed's task picker and can be assigned keyboard shortcuts.

## Writing effective records

Prefer:

> We expected SQLite paging to reduce startup work. Release traces showed
> renderer grouping was the bottleneck, while paging introduced sequential
> catch-up requests. Keep the compact read model unless new measurements exceed
> the recorded budget.

Avoid:

> Tried paging. It didn't work.

An effective record contains:

- the original hypothesis;
- the change attempted;
- the observed result;
- concrete evidence;
- the conclusion;
- when the conclusion should be reconsidered.

## Current limitations

Tincan does not yet:

- import agent transcripts;
- query ACP sessions;
- generate summaries with an AI model;
- compute a code dependency graph;
- perform semantic search;
- update changelogs;
- create commits, PRs, tags, or releases;
- publish field notes.

These can be added as optional workflows without changing the Markdown source
of truth.
