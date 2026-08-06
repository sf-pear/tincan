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

After the CLI is updated, the next interactive Tincan command lists stale
installed skills and offers to update them before continuing. Non-interactive
commands never prompt.

`init` creates `.tincan/` and excludes it from normal Git tracking through the
repository's local exclude file.

## Commands

| Command | What it does | Why use it |
| --- | --- | --- |
| `init [PATH]` | Creates private `.tincan/` storage. | Start using Tincan in a project. |
| `journal` | Adds meaningful progress, questions, or next steps to today's journal. | Preserve short-term continuity without recording every action. |
| `resume` | Prints the latest journal. | Recover current work and open matters at the start of a session. |
| `decide STATEMENT` | Creates an accepted decision with optional file and topic links. | Preserve a choice that should constrain future work. |
| `learn STATEMENT` | Creates an evidence-supported learning. | Reuse a durable conclusion established by real evidence. |
| `search QUERY` | Searches headings, metadata, and Markdown bodies. | Find relevant memory without knowing a record ID. |
| `show UUID` | Prints one complete decision or learning. | Load a specific record returned by `search` or `changes`. |
| `summary [PATH]` | Counts stored decisions, learnings, and journals. | See what memory exists; add `--verbose` for headings and paths. |
| `changes` | Matches Git-changed files to records linked through `--file`. | Let an agent recover relevant constraints without guessing search terms. |
| `skill install` | Installs the bundled workflow into selected agent harnesses. | Have agents maintain and consult Tincan during development. |

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
tincan changes
```

`decide` and `learn` create UUID-named Markdown files with validated
frontmatter. Add detailed reasoning directly to the Markdown body.

Run `tincan --help` for complete syntax and options.
