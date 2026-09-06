---
name: tincan
description: Proactively maintain focused development continuity in workspaces containing `.tincan/config.toml`, including during ordinary implementation, debugging, and design discussion—not only explicit resume or wrap-up requests. Preserve intended outcomes, meaningful handoffs, accepted decisions, evidence-supported learnings, and user-approved ideas for later without recording routine work.
---

# Tincan

Treat Markdown under `.tincan/` as canonical private project memory. A Tincan
workspace may contain zero, one, or several Git repositories.

Maintain Tincan as meaningful context emerges; do not wait for an explicit
wrap-up. Record only context whose loss could send later work in the wrong
direction, repeat a disproven approach, violate an accepted constraint, or lose
an idea the user wanted preserved.

Keep the kinds of memory distinct:

- The plan is the small set of outcomes the user currently intends to pursue.
- The later shelf contains possibilities worth remembering, not commitments or
  implied direction.
- The journal is the latest handoff: meaningful progress, open questions, and
  the next useful starting point.
- Decisions are accepted choices that constrain future work.
- Learnings are evidence-supported conclusions that should change future work.

## Start work

1. Find `.tincan/config.toml` in the current directory or an ancestor. If none
   exists, offer to run `tincan init <directory>` through the harness's standard
   user-question tool with `Initialize Tincan` and `Not now` choices. Never run
   `tincan init` without the user's explicit confirmation.
2. Run `tincan resume` once to read the living plan, later shelf, and latest
   journal. Do not
   repeat startup commands during the same task when that context is still
   available. If `tincan` is unavailable, suggest `cargo install tincan-cli`
   and continue without Tincan.
3. Run `tincan changes` when existing file-linked memory may matter.
4. After understanding the current task, search focused terms with
   `tincan search`. Search covers project records and global learnings. Load
   only relevant full records with `show`, and do not repeat searches already
   performed for the same task.

## Review history

When the user asks for a project retrospective or a review of work over time,
run bare `tincan review` first to inspect the recorded date span and yearly
counts. Then request only the useful calendar context with `review --year
YEAR`, `review --month YYYY-MM`, `review --quarter YYYY-QN`, or `review --half
YYYY-HN`; use `review --all-time`
only when the overview shows that loading every record is reasonable or the
user explicitly requests all history. Add `--all-projects` only when the user
requests a cross-project review.

If a cross-project review reports unavailable workspaces, do not remove them
automatically because a path may have moved or be temporarily disconnected.
Use `tincan projects` to show registry state. When the user asks to clean it up,
use `tincan projects unregister PATH_OR_ID`. Unregistering changes only personal
discovery state and never deletes project `.tincan/` memory; rerunning `tincan
init PATH` explicitly registers that workspace again. If Tincan reports the
same workspace ID at two live paths, ask the user which location is the original
instead of treating the copy as a move. With approval, run `tincan init PATH` on
the copy to assign it a new ID and register it separately.

Tincan returns deterministic source context, not an interpretation. Summarize
that context for the user's stated purpose, distinguish recorded evidence from
inference, and do not store a derived review unless the user asks for a file.

## Keep the skill current

Run `tincan skill status` when the user asks whether installed Tincan skills are
current or when troubleshooting differing behavior between agents. The check is
read-only. If it reports an available update, ask the user to run
`tincan skill install`; do not modify user-wide skill installations without
their confirmation.

## Maintain the plan

Edit `.tincan/plan.md` directly. Keep only current outcome-level work.
Do not turn it into an implementation checklist or duplicate an issue tracker.
Remove completed items. Preserve only any resulting handoff context that would
otherwise be lost.

After meaningful project work, review the plan before handing control back.
Change it only when the work completes, invalidates, refines, or reveals an
outcome. The plan guides current project direction; routine activity
does not require a plan edit.

## Maintain the later shelf

Use `tincan remember <text>` for a small observation, possible idea, or minor
issue that may be worth revisiting but is not an intended outcome, accepted
decision, or evidence-supported learning. Use it without another confirmation
when the user explicitly says to remember, save, revisit, or consider something
later. If preserving the thought is the agent's inference, ask a lightweight
question such as “Worth putting on the later shelf?” first. Do not record raw
brainstorming or every passing possibility.

`tincan resume` includes the shelf so possible ideas can be compared with the
current request. Do not implement, plan, or present an item as current direction
merely because it appears there. If an item seems relevant, mention the
connection and ask before folding it into the current work unless the user has
already directed that change. Use `tincan later` to view the shelf again without
reloading the rest of the handoff.

Keep `.tincan/later.md` short by removing discarded items and removing items
promoted into the plan, a decision, or a learning. Tentative items need no audit
trail, priority, status, assignment, or due date.

## Maintain the handoff

Before handing control back after meaningful project work, briefly review
whether the next person or agent still has enough context to resume safely.
Preserve context that would otherwise remain in someone's head: where work
stopped, what is unresolved, why the current state matters, and the most useful
next starting point.

If the handoff materially changed, update today's journal with only the
applicable `--done`, `--planned`, `--question`, and `--next` bullets. If the
existing journal and plan already provide enough context, write nothing. Do not
wait for the user to request a wrap-up, and do not let memory maintenance delay
the requested result.

The journal is a sufficient handoff, not a complete history. Do not mirror
commits, changelogs, issue trackers, file lists, routine commands, or the final
response. A push, merge, release, or other milestone can prompt a handoff
review, but does not itself require a journal entry.

## Classify memory

- Record an accepted choice with `tincan decide` when the user clearly commits
  to it. If acceptance is ambiguous or the classification requires judgment,
  ask before recording it.
- Journal bullets preserve only the meaningful progress, unfinished context,
  open questions, and concrete next starting point needed for a useful handoff.
  Include transient status only when it materially explains a blocker, an open
  question, why the current state matters, or where work should resume.
- Use `--supersedes <uuid>` when a decision replaces an active decision.
- `tincan learn <statement>` records an evidence-supported conclusion that
  remains useful beyond the current session. It must change a future
  implementation, debugging, review, or architectural action: what concrete
  mistake, regression, or repeated investigation will it prevent?

Add one or two `--topic` values when a learning has an obvious useful domain,
such as `react`, `rust`, or `accessibility`; do not invent or enforce a taxonomy.
Lifted global learnings inherit the project learning's topics.

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
   outcomes, accepted decisions, evidence-supported learnings, remaining
   direction, open questions, and the next starting point.
2. Record any missed durable decisions or learnings. Do not invent acceptance
   or evidence.
3. Update `.tincan/plan.md` so it reflects only current direction.
4. Update today's journal only where the handoff materially changed. Keep
   bullets short and do not duplicate Git history, decisions, or learnings;
   their existing records are canonical.
5. Run `tincan resume` and show the finalized journal to the user.

Distinguish implemented and verified work from work that was only decided or
planned. Do not let bookkeeping block the user's main work.
