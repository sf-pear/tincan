---
name: tincan
description: Maintain private workspace-local development continuity with Tincan's plan, daily journal, accepted decisions, and evidence-supported learnings. Use when a workspace has or could benefit from `.tincan/config.toml`, when resuming work, or when the user asks to wrap up, finish for today, record what was learned, or prepare to continue tomorrow.
---

# Tincan

Treat Markdown under `.tincan/` as canonical private project memory. A Tincan
workspace may contain zero, one, or several Git repositories.

## Start work

1. Confirm `tincan --help` works. If not, suggest `cargo install tincan-cli`
   and continue without Tincan.
2. Find `.tincan/config.toml` in the current directory or an ancestor. If none
   exists, offer to run `tincan init <directory>` through the harness's standard
   user-question tool with `Initialize Tincan` and `Not now` choices. Never run
   `tincan init` without the user's explicit confirmation.
3. Run `tincan plan`, then `tincan resume`.
4. Run `tincan changes` when existing file-linked memory may matter. Search
   focused terms with `tincan search`, and load relevant records with `show`.

## Maintain the plan

Edit `.tincan/plan.md` directly. Keep only current outcome-level work and ideas.
Do not turn it into an implementation checklist or duplicate an issue tracker.
Remove completed items because the journal preserves their history.

## Classify memory

- Journal bullets summarize meaningful completed work, decisions, learnings,
  planned work, open questions, and the concrete next starting point. Include
  transient status only when it materially explains progress, a blocker, an
  open question, or where work should resume.
- `tincan decide <statement>` records an accepted choice that constrains future
  work. Use `--supersedes <uuid>` when replacing an active decision.
- `tincan learn <statement>` records an evidence-supported conclusion that
  remains useful beyond the current session. It must change a future
  implementation, debugging, review, or architectural action: what concrete
  mistake, regression, or repeated investigation will it prevent?

Use workspace-relative `--file` paths. Let Tincan create UUIDs and frontmatter,
then add useful detail below the generated H1. Do not record using Tincan,
maintaining project memory, routine command execution, routine verification with
no durable implication, or transient dependency and audit status. Do not
promote a journal status update into a learning unless it establishes a durable,
reusable conclusion. Do not record routine edits, speculation, raw transcripts,
credentials, or customer data.

## Wrap up

Treat natural requests such as "wrap up", "done for today", "good night",
"what did we learn", and "where do I start tomorrow" as a wrap-up request.

1. Review the conversation, current journal, and plan for meaningful completed
   work, accepted decisions, evidence-supported learnings, remaining plans,
   open questions, and the next starting point.
2. Record any missed durable decisions or learnings. Do not invent acceptance
   or evidence.
3. Update `.tincan/plan.md` so it reflects only current direction.
4. Update today's journal with repeatable `--done`, `--decision`, `--learning`,
   `--planned`, `--question`, and `--next` options. Keep bullets short.
5. Run `tincan resume` and show the finalized journal to the user.

Distinguish implemented and verified work from work that was only decided or
planned. Do not let bookkeeping block the user's main work.
