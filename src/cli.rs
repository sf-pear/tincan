use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Init { repo: PathBuf },
    Summary { repo: PathBuf, verbose: bool },
    Record(RecordArgs),
    Journal(JournalArgs),
    Resume { repo: PathBuf },
    Search { repo: PathBuf, query: String },
    Show { repo: PathBuf, id: String },
    Check { repo: PathBuf },
    SkillInstall { path: Option<PathBuf>, force: bool },
}

#[derive(Debug, PartialEq)]
pub struct JournalArgs {
    pub repo: PathBuf,
    pub done: Vec<String>,
    pub questions: Vec<String>,
    pub next: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct RecordArgs {
    pub kind: String,
    pub repo: PathBuf,
    pub statement: String,
    pub files: Vec<String>,
    pub topics: Vec<String>,
    pub evidence: Vec<String>,
    pub related: Vec<String>,
    pub supersedes: Vec<String>,
}

pub fn parse(args: Vec<String>) -> Result<Command, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "-V" | "--version" | "version" => Ok(Command::Version),
        "init" => Ok(Command::Init {
            repo: positional_repo(&args[1..])?,
        }),
        "summary" => parse_summary(&args[1..]),
        "decide" => parse_record("decision", &args[1..]),
        "learn" => parse_record("learning", &args[1..]),
        "journal" => parse_journal(&args[1..]),
        "resume" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if !values.positionals.is_empty() {
                return Err("resume does not accept positional arguments".to_string());
            }
            Ok(Command::Resume {
                repo: values.directory()?,
            })
        }
        "search" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if values.positionals.len() > 1 {
                return Err("search accepts exactly one query".to_string());
            }
            let query = values
                .positionals
                .first()
                .cloned()
                .ok_or_else(|| "search requires a query".to_string())?;
            Ok(Command::Search {
                repo: values.directory()?,
                query,
            })
        }
        "show" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if values.positionals.len() > 1 {
                return Err("show accepts exactly one record ID".to_string());
            }
            let id = values
                .positionals
                .first()
                .cloned()
                .ok_or_else(|| "show requires a record ID".to_string())?;
            Ok(Command::Show {
                repo: values.directory()?,
                id,
            })
        }
        "check" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if !values.positionals.is_empty() {
                return Err("check does not accept positional arguments".to_string());
            }
            Ok(Command::Check {
                repo: values.directory()?,
            })
        }
        "skill" => parse_skill(&args[1..]),
        other => Err(format!("unknown command: {other}\n\n{}", help())),
    }
}

fn parse_journal(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&["directory", "done", "question", "next"])?;
    values.ensure_at_most_one(&["directory"])?;
    if !values.positionals.is_empty() {
        return Err("journal does not accept positional arguments".to_string());
    }
    let done = values.many("done");
    let questions = values.many("question");
    let next = values.many("next");
    if done.is_empty() && questions.is_empty() && next.is_empty() {
        return Err(
            "journal requires at least one --done, --question, or --next bullet".to_string(),
        );
    }
    Ok(Command::Journal(JournalArgs {
        repo: values.directory()?,
        done,
        questions,
        next,
    }))
}

fn parse_summary(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&["verbose"])?;
    values.ensure_at_most_one(&["verbose"])?;
    if values.positionals.len() > 1 {
        return Err("summary accepts at most one repository path".to_string());
    }
    Ok(Command::Summary {
        repo: values
            .positionals
            .first()
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|| std::env::current_dir().map_err(|error| error.to_string()))?,
        verbose: values.present("verbose"),
    })
}

fn parse_skill(args: &[String]) -> Result<Command, String> {
    if args.first().map(String::as_str) != Some("install") {
        return Err("skill requires `install`".to_string());
    }
    let values = Flags::parse(&args[1..])?;
    values.ensure_only(&["path", "force"])?;
    values.ensure_at_most_one(&["path", "force"])?;
    if !values.positionals.is_empty() {
        return Err("skill install does not accept positional arguments".to_string());
    }
    Ok(Command::SkillInstall {
        path: values.one("path").map(PathBuf::from),
        force: values.present("force"),
    })
}

fn parse_record(kind: &str, args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&[
        "directory",
        "file",
        "topic",
        "evidence",
        "related",
        "supersedes",
    ])?;
    values.ensure_at_most_one(&["directory"])?;
    if values.positionals.len() != 1 {
        return Err(format!(
            "{} requires exactly one concise statement",
            if kind == "decision" {
                "decide"
            } else {
                "learn"
            }
        ));
    }
    record_command(
        kind.to_string(),
        values.directory()?,
        values.positionals[0].clone(),
        &values,
    )
}

fn record_command(
    kind: String,
    repo: PathBuf,
    statement: String,
    values: &Flags,
) -> Result<Command, String> {
    let statement = statement.trim().to_string();
    if statement.is_empty() || statement.contains(['\n', '\r']) {
        return Err("record statement must be one non-empty line".to_string());
    }
    if kind != "decision" && values.present("supersedes") {
        return Err("remove --supersedes: it is only valid for decisions".to_string());
    }
    for option in ["related", "supersedes"] {
        for id in values.many(option) {
            if Uuid::parse_str(&id).is_err() {
                return Err(format!("--{option} requires a valid UUID: {id}"));
            }
        }
    }
    Ok(Command::Record(RecordArgs {
        kind,
        repo,
        statement,
        files: values.many("file"),
        topics: values.many("topic"),
        evidence: values.many("evidence"),
        related: values.many("related"),
        supersedes: values.many("supersedes"),
    }))
}

fn positional_repo(args: &[String]) -> Result<PathBuf, String> {
    if args.is_empty() {
        std::env::current_dir().map_err(|error| error.to_string())
    } else if args.len() == 1 {
        Ok(PathBuf::from(&args[0]))
    } else {
        Err("expected zero or one repository path".to_string())
    }
}

#[derive(Default)]
struct Flags {
    values: Vec<(String, Option<String>)>,
    positionals: Vec<String>,
}

impl Flags {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut index = 0;
        while index < args.len() {
            let value = &args[index];
            if let Some(name) = value.strip_prefix("--") {
                if name.is_empty() {
                    return Err("empty flag".to_string());
                }
                if matches!(name, "changed" | "force" | "verbose") {
                    parsed.values.push((name.to_string(), None));
                    index += 1;
                    continue;
                }
                let next = args
                    .get(index + 1)
                    .filter(|candidate| !is_option(candidate))
                    .cloned()
                    .ok_or_else(|| format!("--{name} requires a value"))?;
                parsed.values.push((name.to_string(), Some(next)));
                index += 2;
            } else if let Some(name) = short_option_name(value) {
                if name == "verbose" {
                    parsed.values.push((name.to_string(), None));
                    index += 1;
                    continue;
                }
                let next = args
                    .get(index + 1)
                    .filter(|candidate| !is_option(candidate))
                    .cloned()
                    .ok_or_else(|| format!("{value} requires a value"))?;
                parsed.values.push((name.to_string(), Some(next)));
                index += 2;
            } else if value.starts_with('-') {
                return Err(format!(
                    "unknown option {value}; run `tincan --help` and retry with a listed option"
                ));
            } else {
                parsed.positionals.push(value.clone());
                index += 1;
            }
        }
        Ok(parsed)
    }

    fn one(&self, name: &str) -> Option<String> {
        self.values
            .iter()
            .rev()
            .find(|(key, _)| key == name)
            .and_then(|(_, value)| value.clone())
    }

    fn many(&self, name: &str) -> Vec<String> {
        self.values
            .iter()
            .filter(|(key, _)| key == name)
            .filter_map(|(_, value)| value.clone())
            .collect()
    }

    fn present(&self, name: &str) -> bool {
        self.values.iter().any(|(key, _)| key == name)
    }

    fn ensure_only(&self, allowed: &[&str]) -> Result<(), String> {
        if let Some((name, _)) = self
            .values
            .iter()
            .find(|(name, _)| !allowed.contains(&name.as_str()))
        {
            return Err(format!(
                "unknown option --{name}; run `tincan --help` and retry with a listed option"
            ));
        }
        Ok(())
    }

    fn ensure_at_most_one(&self, names: &[&str]) -> Result<(), String> {
        for name in names {
            if self.values.iter().filter(|(key, _)| key == name).count() > 1 {
                return Err(format!(
                    "--{name} may only be provided once; keep the intended value and retry"
                ));
            }
        }
        Ok(())
    }

    fn directory(&self) -> Result<PathBuf, String> {
        self.one("directory")
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|| std::env::current_dir().map_err(|error| error.to_string()))
    }
}

fn short_option_name(value: &str) -> Option<&'static str> {
    match value {
        "-d" => Some("directory"),
        "-v" => Some("verbose"),
        _ => None,
    }
}

fn is_option(value: &str) -> bool {
    value.starts_with("--") || short_option_name(value).is_some()
}

pub fn help() -> &'static str {
    r#"Tincan: a plain-Markdown development journal

Tincan keeps a private daily journal plus durable decisions and learnings under
the repository's .tincan/ directory. Markdown remains the source of truth.

USAGE
  tincan <COMMAND> [OPTIONS]

COMMANDS
  init [REPOSITORY]             Initialize private .tincan/ storage
  summary [REPOSITORY] [-v|--verbose]
                                Count stored memory; optionally list headings
  journal [OPTIONS]             Add completed work, questions, or next steps
  resume [-d|--directory PATH]  Print the latest daily journal
  decide STATEMENT [OPTIONS]    Create an accepted decision record
  learn STATEMENT [OPTIONS]     Create an evidence-supported learning record
  search [-d|--directory PATH] QUERY
                                List matching records and their IDs
  show [-d|--directory PATH] RECORD_ID
                                Print one record; use an ID returned by search
  check [-d|--directory PATH]   Find records related to Git-changed paths
  skill install [OPTIONS]       Install the bundled Agent Skill
  help, --help                  Print this help
  version, --version            Print the installed Tincan version

REPOSITORIES AND FILES
  Commands use the current Git repository unless REPOSITORY or --directory is
  given. -d is the short form of --directory. Tincan writes Markdown only under
  .tincan/ and excludes it from normal Git tracking by default. You may also
  open those Markdown files directly.

RECORD IDS
  Decisions and learnings receive stable UUID v7 IDs such as
  019c4ea8-7e42-7b31-a211-8df9357d747c. `decide` and `learn` print the new ID;
  `search` prints matching IDs. `show` is a convenience for agents and scripts.

EXAMPLES
  tincan init .
  tincan journal --done "Implemented deterministic path matching"
  tincan decide "Keep Markdown canonical" --topic storage
  tincan learn "Paging did not reduce rendering work" --evidence "Release trace"
  tincan search "markdown"
  tincan show 019c4ea8-7e42-7b31-a211-8df9357d747c

JOURNAL OPTIONS
  tincan journal [-d|--directory PATH] [--done TEXT ...]
                 [--question TEXT ...] [--next TEXT ...]

  At least one journal bullet is required. Each bullet option is repeatable.

SKILL INSTALL
  tincan skill install [--path SKILLS_DIRECTORY] [--force]

  Without --path, detect user-wide Agent Skills harnesses and select all by
  default. Use arrow keys to move, Space to select or unselect, A to toggle all
  or none, Enter to continue, and Escape to cancel. Every installation requires
  answering a final y/N confirmation and pressing Enter. Use --path for an
  explicit, non-interactive or project-local destination.

RECORD OPTIONS
  tincan decide STATEMENT [OPTIONS]
  tincan learn STATEMENT [OPTIONS]

  -d, --directory PATH
  --file REPOSITORY_PATH         repeatable
  --topic TEXT                   repeatable
  --evidence TEXT                repeatable
  --related UUID                 repeatable
  --supersedes UUID              repeatable; decisions only

  Tincan creates valid frontmatter and a UUID filename, then prints the path.
  Add detailed reasoning directly to the Markdown body below its heading.
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repeated_record_fields() {
        let command = parse(
            [
                "learn",
                "Paging did not reduce rendering work",
                "--file",
                "src/a.rs",
                "--file",
                "src/b.rs",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .unwrap();

        let Command::Record(record) = command else {
            panic!("expected record command");
        };
        assert_eq!(record.kind, "learning");
        assert_eq!(record.statement, "Paging did not reduce rendering work");
        assert_eq!(record.files, vec!["src/a.rs", "src/b.rs"]);
    }

    #[test]
    fn gives_actionable_record_validation_errors() {
        let learning_status = parse(
            ["learn", "Paging", "--status", "accepted"]
                .map(str::to_string)
                .to_vec(),
        )
        .unwrap_err();
        assert_eq!(
            learning_status,
            "unknown option --status; run `tincan --help` and retry with a listed option"
        );

        let typo = parse(
            ["decide", "Paging", "--files", "src/a.rs"]
                .map(str::to_string)
                .to_vec(),
        )
        .unwrap_err();
        assert!(typo.contains("unknown option --files"));

        assert!(
            parse(["learn", "one", "two"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("exactly one concise statement")
        );
        assert!(
            parse(
                ["learn", "A fact", "--supersedes", "id"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap_err()
            .contains("only valid for decisions")
        );
    }

    #[test]
    fn parses_skill_install_options() {
        let command = parse(
            ["skill", "install", "--path", "agent-skills", "--force"]
                .map(str::to_string)
                .to_vec(),
        )
        .unwrap();

        assert_eq!(
            command,
            Command::SkillInstall {
                path: Some(PathBuf::from("agent-skills")),
                force: true,
            }
        );
    }

    #[test]
    fn parses_daily_journal_bullets() {
        let command = parse(
            [
                "journal",
                "--done",
                "Implemented search",
                "--question",
                "Should topics be normalized?",
                "--next",
                "Test on another repository",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .unwrap();

        let Command::Journal(journal) = command else {
            panic!("expected journal command");
        };
        assert_eq!(journal.done, vec!["Implemented search"]);
        assert_eq!(journal.questions, vec!["Should topics be normalized?"]);
        assert_eq!(journal.next, vec!["Test on another repository"]);
    }

    #[test]
    fn parses_summary_without_an_inspect_alias() {
        assert!(matches!(
            parse(vec!["summary".to_string()]).unwrap(),
            Command::Summary { verbose: false, .. }
        ));
        assert!(matches!(
            parse(["summary", "-v"].map(str::to_string).to_vec()).unwrap(),
            Command::Summary { verbose: true, .. }
        ));
        assert!(matches!(
            parse(
                ["summary", "project", "--verbose"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Summary { repo, verbose: true } if repo == std::path::Path::new("project")
        ));
        assert!(parse(vec!["inspect".to_string()]).is_err());
    }

    #[test]
    fn parses_directory_and_rejects_the_old_repo_option() {
        assert!(matches!(
            parse(
                ["resume", "-d", "project"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Resume { repo } if repo == std::path::Path::new("project")
        ));
        assert!(matches!(
            parse(
                ["resume", "--directory", "project"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Resume { repo } if repo == std::path::Path::new("project")
        ));
        assert!(
            parse(["resume", "--repo", "project"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("unknown option --repo")
        );
    }

    #[test]
    fn parses_version_flags() {
        for argument in ["-V", "--version", "version"] {
            assert_eq!(parse(vec![argument.to_string()]).unwrap(), Command::Version);
        }
    }

    #[test]
    fn rejects_extra_arguments_and_unknown_options() {
        assert_eq!(
            parse(["search", "one", "two"].map(str::to_string).to_vec()).unwrap_err(),
            "search accepts exactly one query"
        );
        assert!(
            parse(
                ["skill", "install", "--unknown", "value"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap_err()
            .contains("unknown option --unknown")
        );
        assert_eq!(
            parse(vec!["check".to_string()]).unwrap(),
            Command::Check {
                repo: std::env::current_dir().unwrap(),
            }
        );
        assert!(
            parse(["check", "--changed"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("unknown option --changed")
        );
    }

    #[test]
    fn help_explains_record_ids_and_direct_markdown_access() {
        assert!(help().contains("use an ID returned by search"));
        assert!(help().contains("open those Markdown files directly"));
        assert!(help().contains("UUID v7 IDs"));
        assert!(help().contains("tincan decide STATEMENT"));
        assert!(!help().contains("record decision"));
    }
}
