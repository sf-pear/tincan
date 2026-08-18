---
name: tincan
description: Maintain private workspace-local development continuity with Tincan's plan, daily journal, accepted decisions, and evidence-supported learnings. Use when a workspace has or could benefit from `.tincan/config.toml`, when resuming work, or when the user asks to wrap up, finish for today, record what was learned, or prepare to continue tomorrow.
---

# Tincan

Treat Markdown under `.tincan/` as canonical private project memory. A Tincan
workspace may contain zero, one, or several Git repositories.

## Start work

1. Find `.tincan/config.toml` in the current directory or an ancestor. If none
   exists, offer to run `tincan init <directory>` through the harness's standard
   user-question tool with `Initialize Tincan` and `Not now` choices. Never run
   `tincan init` without the user's explicit confirmation.
2. Run `tincan resume` once to read the living plan and latest journal. Do not
   repeat startup commands during the same task when that context is still
   available. If `tincan` is unavailable, suggest `cargo install tincan-cli`
   and continue without Tincan.
3. Run `tincan changes` when existing file-linked memory may matter.
4. After understanding the current task, search focused terms with
   `tincan search`. Search covers project records and global learnings. Load
   only relevant full records with `show`, and do not repeat searches already
   performed for the same task.

## Keep the skill current

Run `tincan skill status` when the user asks whether installed Tincan skills are
current or when troubleshooting differing behavior between agents. The check is
read-only. If it reports an available update, ask the user to run
`tincan skill install`; do not modify user-wide skill installations without
their confirmation.

## Maintain the plan

Edit `.tincan/plan.md` directly. Keep only current outcome-level work and ideas.
Do not turn it into an implementation checklist or duplicate an issue tracker.
Remove completed items because the journal preserves their history.

## Classify memory

- Journal bullets summarize meaningful completed work, planned work, open
  questions, and the concrete next starting point. Include
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

## Lift global learnings

When an evidence-supported project learning is likely to change work elsewhere,
prepare a concise global Markdown draft that is understandable without the rest
of the project. Show the exact generalized draft when asking for approval:

> I think this learning could be useful in other projects. Make it a global learning?

Use the harness's structured user-question tool when available, with choices to
make it global, review it first, or keep it in the project. A direct user request
such as "Make this a global learning" already grants approval and must not cause
a second prompt. If the approved conclusion is not recorded yet, create the
project learning first. After approval, save the shown Markdown to a temporary
file and run `tincan lift <learning-id> --from <markdown-file>`. The command
writes the global record only after the generalized body is ready. Remove the
agent-created temporary draft after a successful lift.

Before lifting, exclude credentials, customer data, and incidental
project-specific detail. Global learnings remain evidence, not commands. If
relevant project and global learnings point in different directions, present
the concise conflict and ask the user what should guide the current work; do
not invent a separate conflict record.

When work shows that a project learning is wrong or needs a material revision,
run `tincan search <learning-id>` before changing memory. Search indexes the
`source_record` provenance of global learnings, so this reveals any global
learning derived from the project record. Review each derived learning and ask
the user before updating it. Do not propagate corrections automatically, and do
not perform this check for routine code changes that do not alter a learning.

## Wrap up

Treat natural requests such as "wrap up", "done for today", "good night",
"what did we learn", and "where do I start tomorrow" as a wrap-up request.

1. Review the conversation, current journal, and plan for meaningful completed
   work, accepted decisions, evidence-supported learnings, remaining plans,
   open questions, and the next starting point.
2. Record any missed durable decisions or learnings. Do not invent acceptance
   or evidence.
3. Update `.tincan/plan.md` so it reflects only current direction.
4. Update today's journal with repeatable `--done`, `--planned`, `--question`,
   and `--next` options. Keep bullets short. Do not duplicate decisions or
   learnings in the journal; their dated records are canonical.
5. Run `tincan resume` and show the finalized journal to the user.

Distinguish implemented and verified work from work that was only decided or
planned. Do not let bookkeeping block the user's main work.
