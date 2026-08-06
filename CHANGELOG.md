# Changelog

All notable changes to Tincan will be documented in this file.

The format follows Keep a Changelog and versions follow Semantic Versioning.

## [Unreleased]

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
