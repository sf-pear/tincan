# Changelog

All notable changes to Tincan will be documented in this file.

The format follows Keep a Changelog and versions follow Semantic Versioning.

## [Unreleased]

### Added

- Initialize portable `.tincan/` ledgers in any Git repository.
- Record decisions and learnings as Markdown notes.
- Enforce managed decision statuses while keeping learnings status-free.
- Supersede decisions without erasing their original reasoning.
- Scan compact record frontmatter before loading relevant full Markdown with
  `tincan show`.
- Search all Tincan records without a database.
- Match changed files against related historical records.
- Create an agent workflow guide and link it from root `AGENTS.md` during
  `tincan init`.
- Bundle a portable Tincan agent skill and install it into Codex or a selected
  harness skills directory with `tincan skill install`.
