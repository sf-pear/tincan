---
name: tincan
description: Maintain private, repository-local development continuity with Tincan's daily journal, accepted decisions, and evidence-supported learnings. Use when working in a Git checkout where Tincan memory exists or could help, including when `.tincan/config.toml` is absent and initialization should be offered, when resuming prior work, when meaningful implementation progress or open work should be journaled, or when durable reasoning should remain available to future people and agents.
---

# Tincan

Treat Markdown under `.tincan/` as canonical private memory for this checkout.
Treat transcripts and conversation summaries as evidence, not accepted truth.

## Start work

1. Read `AGENTS.md` when present.
2. Confirm Tincan is available with `tincan --help`. If the command is missing,
   briefly explain that the skill is installed but the CLI is not, suggest
   `cargo install tincan-cli`, and continue the user's work without Tincan.
3. Check for `.tincan/config.toml`. If it is absent, offer to initialize Tincan
   in the current Git repository. Prefer the harness's structured user-question
   tool when one is available, with clear `Initialize Tincan` and `Not now`
   choices. Do not assume consent. If the user chooses initialization, run
   `tincan init <repository>`. If they decline, do not answer, or the session is
   non-interactive, continue without Tincan and do not ask again during the same
   task.
4. After confirming or creating `.tincan/config.toml`, read
   `.tincan/AGENT_GUIDE.md`.
5. Run `tincan resume` to read the latest daily journal before continuing prior
   work.
6. Run `tincan check` when touching an area with existing memory.
7. Search focused files, features, or concepts with `tincan search "<query>"`.
8. Load only relevant full records with `tincan show <record-id>`.

Never run `tincan init` without the user's explicit confirmation.

## Classify before writing

Use this test:

- **Journal:** Does this describe what was implemented today, an unresolved
  question, or concrete work still to do? Add one concise `--done`, `--question`,
  or `--next` bullet. Do not put full reasoning here.
- **Learning:** Did evidence establish knowledge that will remain useful beyond
  today's work? Record the conclusion, evidence, scope, and reconsideration
  conditions. A failed approach qualifies only when it taught something durable.
- **Decision:** Was a choice accepted that constrains future work? Record the
  choice, reasoning, consequences, and constraints. Do not record proposals or
  open questions as decisions.

A meaningful completed change may produce a short journal bullet plus a decision
or learning. Avoid repeated prose: the journal records progress; the durable
record holds the reasoning.

## Use the commands

Update the one living journal file for the local calendar day as work develops:

```text
tincan journal --done <implemented item>
tincan journal --question <unresolved question>
tincan journal --next <unfinished item>
```

Add bullets after meaningful developments, not after routine edits or commands.
Tincan ignores exact duplicates. If the exact text of a `Next` bullet is later
added as `Done`, Tincan moves it rather than retaining stale duplication.

- Run `tincan decide <statement>` only after a choice is accepted. Use
  `--supersedes <uuid>` when it replaces an earlier decision. Do not pass a
  status; Tincan creates decisions as `active` and manages supersession.
- Run `tincan learn <statement>` only when evidence supports a reusable
  conclusion. Learnings do not have a status.
- Use repository-relative `--file` values, focused `--topic` values, concrete
  `--evidence`, and `--related` UUIDs where useful.

Use only these record shapes:

```text
tincan decide <statement> [--file <path>] [--topic <text>] [--evidence <text>] [--related <uuid>] [--supersedes <uuid>]
tincan learn <statement> [--file <path>] [--topic <text>] [--evidence <text>] [--related <uuid>]
```

Repeat bracketed metadata options when needed. Tincan prints the created UUID
and Markdown path. Add detailed context, reasoning, consequences, scope, and
reconsideration conditions directly beneath the H1 in that file. Do not edit
the generated frontmatter or invent another record type or option.

Do not create decisions or learnings for routine edits, progress updates,
speculation, todos, or facts already obvious from the code. Put meaningful
progress and open work in the journal instead.

If Tincan rejects a record, follow the corrective error, fix the arguments, and
retry. Always let Tincan create the UUID and frontmatter; edit only the Markdown
body directly afterward.

## Maintain truth and privacy

- Never convert a suggestion into an accepted decision without confirmation.
- Record only evidence and verification that actually exist.
- Never store credentials, access tokens, customer data, or raw transcripts.
- Do not let bookkeeping block the requested development work; report failures
  and continue safely when possible.

## Finish work

After meaningful work, ensure its journal bullets reflect what is done, what is
still open, and what should happen next. Record accepted decisions and durable,
evidence-supported learnings when warranted. Mention created or updated Tincan
artifacts in the final response. Do not use Tincan as a general task tracker or
publishing tool.
