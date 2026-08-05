---
name: tincan
description: Preserve durable, repository-local development memory with the Tincan CLI. Use when working in a Git repository containing `.tincan/config.toml`, when asked to use Tincan, or when recording an important attempt, accepted decision, evidence-supported learning, substantial session, or handoff for future people and agents.
---

# Tincan

Use Tincan to preserve the engineering record that should survive changing
agents, tools, and conversations. Treat Markdown under `.tincan/` as canonical
project memory and raw transcripts as evidence rather than accepted truth.

## Start work

1. Read the repository's `AGENTS.md` and `.tincan/AGENT_GUIDE.md`.
2. Confirm the CLI is available with `tincan --help`. If it is unavailable,
   report that briefly and continue the task without inventing Tincan records.
3. Run `tincan check --changed` before changing an area with existing work.
4. Search focused terms with `tincan search "<file, feature, or concept>"` when
   the task may overlap earlier work.
5. Read matching records before choosing an approach.

Do not initialize Tincan unless the user asks to set it up. `tincan init`
creates `.tincan/` and may add a pointer to the repository's root `AGENTS.md`.

## Preserve durable memory

Create records only when they reduce future uncertainty. Do not record routine
edits, conversational speculation, or facts that are obvious from the code.

- Record an `attempt` when a meaningful approach succeeds, fails, is
  inconclusive, or is superseded. Include the hypothesis, observed result,
  evidence, affected paths or topics, and the conclusion for future work.
- Record a `decision` only after a choice is accepted. State the reason,
  consequences, affected areas, and related evidence or attempt IDs.
- Record a `learning` only when evidence supports a reusable conclusion. State
  where it applies and when it should be reconsidered.
- Use `capture` after a substantial session. Treat the generated file as a
  draft and replace prompts with verified outcomes and checks that actually ran.
- Use `handoff` when meaningful work remains or another person or agent will
  continue. Give exact next actions, accepted decisions, risks, verification
  gaps, and relevant working-tree state.

Prefer repository-relative values for `--affects`. Link records with
`--related` when one justifies another. Preserve failed and superseded attempts
as distinct records rather than rewriting history.

## Maintain truth and privacy

- Never turn a user idea or transcript statement into an accepted decision
  without confirmation from the work or the user.
- Record only verification that actually ran and evidence that actually exists.
- Never store credentials, access tokens, private customer data, or complete raw
  transcripts.
- Keep draft sessions and field notes private when they contain unreviewed or
  sensitive material.
- Do not let Tincan bookkeeping block the requested development work. If a
  command fails, report the failure and continue safely when possible.

## Finish work

1. Run the relevant project verification.
2. Record only the important attempts, accepted decisions, and reusable
   learnings produced by the work.
3. Capture a substantial session or create a handoff when warranted.
4. Mention created Tincan records in the final response so the user can review
   them.
