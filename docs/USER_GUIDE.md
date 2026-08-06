# Tincan User Guide

Tincan keeps important development memory in repository-local Markdown.
Developers can record and review it directly. During agent-assisted work, the
installed skill guides the agent to keep it current.

Tincan preserves three kinds of memory:

- **Journal:** meaningful progress, open questions, and next steps.
- **Decision:** an accepted choice that should guide future work.
- **Learning:** an evidence-supported conclusion worth reusing.

It records what should survive the current session, not every action or
conversation.

## Setup

```powershell
cargo install tincan-cli
tincan skill install
tincan init C:\path\to\project
```

The skill installer detects supported user-wide agent harnesses. Use
`--path <skills-directory>` for another location and `--force` to update an
existing installation.

`init` creates `.tincan/` and excludes it from normal Git tracking through the
repository's local exclude file.

## Commands

| Command | Purpose |
| --- | --- |
| `init [PATH]` | Initialize Tincan in a Git repository. |
| `journal` | Record meaningful progress, questions, and next steps. |
| `resume` | Print the latest journal entry. |
| `decide STATEMENT` | Record an accepted decision. |
| `learn STATEMENT` | Record an evidence-supported learning. |
| `search QUERY` | Search headings, metadata, and Markdown bodies. |
| `show UUID` | Print one decision or learning. |
| `summary [PATH]` | Count decisions, learnings, and journal entries. Add `--verbose` to list their headings. |
| `check` | Find records related to changed files. |
| `skill install` | Install the bundled agent skill. |

Commands use the current directory by default. Pass `-d <path>` or
`--directory <path>` to target another directory.

## Examples

```powershell
tincan journal --done "Implemented deterministic path matching"
tincan journal --question "Should adjacent details be preloaded?"
tincan journal --next "Add the stale-response regression test"

tincan decide "Keep Markdown canonical" --topic storage

tincan learn "Paging did not reduce rendering work" `
  --file src/gallery.rs `
  --evidence "Release-build trace"

tincan resume
tincan search "storage"
tincan show <record-id>
tincan summary
tincan check
```

`decide` and `learn` create UUID-named Markdown files with validated
frontmatter. Add detailed reasoning directly to the Markdown body.

Run `tincan --help` for complete syntax and options.
