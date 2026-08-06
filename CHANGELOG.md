# Changelog

All notable changes to Tincan will be documented in this file.

The format follows Keep a Changelog and versions follow Semantic Versioning.

## [Unreleased]

## [0.3.1] - 2026-08-06

### Fixed

- Normalize Windows workspace and Git repository paths before comparing them,
  avoiding false “outside Git repository” errors during initialization.

### Changed

- Default the final interactive `tincan skill install` confirmation to yes, so
  pressing Enter installs into the selected destinations.

## [0.3.0] - 2026-08-06

### Added

- Add one living `.tincan/plan.md` for outcome-level planned work and ideas.
- Let journals summarize decisions, learnings, planned work, open questions,
  and next steps alongside completed work.

### Changed

- Define a Tincan workspace by its `.tincan/config.toml` rather than a Git root.
- Support non-Git parent workspaces containing multiple nested Git repositories,
  with `changes` aggregating workspace-relative paths across them.
- Teach the bundled skill to reconcile the plan and journal during natural
  end-of-session wrap-ups.

## [0.2.1] - 2026-08-06

### Changed

- Rename `tincan check` to `tincan changes` and group related memory directly
  beside each Git-changed file.
- Explain what each command does and when it is useful in the user guide.
- Keep picker instructions beneath the skill destinations and summarize the
  selected home-relative paths before confirmation.
- Detect outdated installed Agent Skills after a CLI update and offer to update
  them on the next interactive invocation without affecting scripted use.

## [0.2.0] - 2026-08-06

### Changed

- Replace the generic `record` command with `decide <statement>` and
  `learn <statement>`.
- Create durable records with pure UUID v7 identities and UUID filenames while
  keeping type information in validated frontmatter.
- Generate a concise H1 scaffold and let people or agents expand the Markdown
  body directly without duplicating the statement as title metadata.
- Rank deterministic search across exact IDs, headings, metadata, and full
  Markdown bodies, with compact matching excerpts.
- Advance newly initialized repository storage to version 2 for the UUID record
  format.
- Simplify the sole check operation from `tincan check --changed` to
  `tincan check`.
- Rename `tincan inspect` to `tincan summary`.
- Replace `--repo` with `-d, --directory` for running a command against another
  working directory while keeping journal options explicit.
- Expand command help with concepts, examples, record-ID discovery, and the
  distinction between `show` and reading canonical Markdown directly.
- Print the stable record ID after creating a decision or learning.
- Detect existing user-wide Agent Skills harnesses, select all by default in a
  standard arrow-key and Space picker, and require confirmation before writing.
- Show the picker's navigation, selection, continuation, and cancellation keys
  directly above the interactive choices.
- Apply a colorful terminal theme to the interactive destination picker and
  confirmation while keeping command output pipe-friendly.
- Configure Dialoguer's standard confirmation to wait for Enter after a yes or
  no answer instead of resolving on the first keypress.
- Teach the bundled skill to offer repository initialization through a
  harness-provided structured question while requiring explicit consent.
- Add one centralized, color-configurable terminal logo used by help,
  initialization, and skill installation without affecting piped output.
- Keep `--path` as the explicit non-interactive installation route and preflight
  all selected destinations before writing any skill files.
- Add black-box tests for the compiled binary's help, error stream, and failure
  exit behavior.

## [0.1.0] - 2026-08-06

### Added

- Initialize portable `.tincan/` ledgers in any Git repository.
- Keep `.tincan/` private by default through verified Git-local exclusion,
  refuse initialization when the folder already has tracked files, and avoid
  modifying tracked project files.
- Maintain one living daily journal for completed work, open questions, and next
  steps, and resume from the latest entry.
- Record decisions and learnings as Markdown notes.
- Enforce managed decision statuses while keeping learnings status-free.
- Supersede decisions without erasing their original reasoning.
- Scan compact record frontmatter before loading relevant full Markdown with
  `tincan show`.
- Search all Tincan records without a database.
- Match changed files against related historical records.
- Create a private agent workflow guide during `tincan init`.
- Bundle a portable Tincan agent skill and install it into Codex or a selected
  harness skills directory with `tincan skill install`.
