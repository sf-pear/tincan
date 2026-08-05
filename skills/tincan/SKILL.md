---
name: tincan
description: Preserve durable, repository-local development decisions and evidence-supported learnings with the Tincan CLI. Use when working in a Git repository containing `.tincan/config.toml`, when asked to use Tincan, or when accepted reasoning from development work should remain available to future people and agents.
---

# Tincan

Treat Markdown under `.tincan/` as canonical project memory. Treat transcripts
and session summaries as evidence, not accepted truth.

## Start work

1. Read `AGENTS.md` and `.tincan/AGENT_GUIDE.md`.
2. Confirm Tincan is available with `tincan --help`.
3. Run `tincan check --changed` when touching an area with existing work.
4. Search focused files, features, or concepts with `tincan search "<query>"`.
5. Load only relevant full records with `tincan show <record-id>`.

Do not run `tincan init` unless the user asks to initialize the repository.

## Preserve durable memory

- Record a `decision` only after a choice is accepted. Put the choice, reasoning,
  consequences, and relevant constraints in `--note`. Use `--supersedes
  <decision-id>` when it replaces an earlier decision. Do not pass a status;
  Tincan creates decisions as `active` and manages supersession.
- Record a `learning` only when evidence supports a reusable conclusion. Include
  what became known, where it applies, and when it should be reconsidered.
  Learnings do not have a status.
- Use repository-relative `--file` values, focused `--topic` values, concrete
  `--evidence`, and `--related` record IDs where useful.

Use only these record shapes:

```text
tincan record decision --title <text> --note <markdown> [--file <path>] [--topic <text>] [--evidence <text>] [--related <id>] [--supersedes <decision-id>]
tincan record learning --title <text> --note <markdown> [--file <path>] [--topic <text>] [--evidence <text>] [--related <id>]
```

Repeat bracketed metadata options when needed. Never invent another record type
or option.

Do not create records for routine edits, progress updates, speculation, or facts
already obvious from the code. A failed approach belongs in a learning only when
it produced durable knowledge.

If Tincan rejects a record, follow the corrective error, fix the arguments, and
retry. Do not abandon a worthwhile record or bypass validation by writing its
Markdown manually.

## Maintain truth and privacy

- Never convert a suggestion into an accepted decision without confirmation.
- Record only evidence and verification that actually exist.
- Never store credentials, access tokens, customer data, or raw transcripts.
- Do not let bookkeeping block the requested development work; report failures
  and continue safely when possible.

## Finish work

After completing and verifying the requested work, record only accepted
decisions and durable, evidence-supported learnings that will reduce future
uncertainty. Mention created record IDs in the final response so the user can
review them. Do not use Tincan as a task tracker, session log, handoff system, or
publishing tool.
