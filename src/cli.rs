use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Init { repo: PathBuf },
    Inspect { repo: PathBuf },
    Record(RecordArgs),
    Journal(JournalArgs),
    Resume { repo: PathBuf },
    Search { repo: PathBuf, query: String },
    Show { repo: PathBuf, id: String },
    CheckChanged { repo: PathBuf },
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
    pub title: String,
    pub note: String,
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
        "init" => Ok(Command::Init {
            repo: positional_repo(&args[1..])?,
        }),
        "inspect" => Ok(Command::Inspect {
            repo: positional_repo(&args[1..])?,
        }),
        "record" => parse_record(&args[1..]),
        "journal" => parse_journal(&args[1..]),
        "resume" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["repo"])?;
            values.ensure_at_most_one(&["repo"])?;
            if !values.positionals.is_empty() {
                return Err("resume does not accept positional arguments".to_string());
            }
            Ok(Command::Resume {
                repo: values.repo()?,
            })
        }
        "search" => {
            let values = Flags::parse(&args[1..])?;
            let query = values
                .positionals
                .first()
                .cloned()
                .ok_or_else(|| "search requires a query".to_string())?;
            Ok(Command::Search {
                repo: values.repo()?,
                query,
            })
        }
        "show" => {
            let values = Flags::parse(&args[1..])?;
            let id = values
                .positionals
                .first()
                .cloned()
                .ok_or_else(|| "show requires a record ID".to_string())?;
            Ok(Command::Show {
                repo: values.repo()?,
                id,
            })
        }
        "check" => {
            let values = Flags::parse(&args[1..])?;
            if !values.present("changed") {
                return Err("check currently requires --changed".to_string());
            }
            Ok(Command::CheckChanged {
                repo: values.repo()?,
            })
        }
        "skill" => parse_skill(&args[1..]),
        other => Err(format!("unknown command: {other}\n\n{}", help())),
    }
}

fn parse_journal(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&["repo", "done", "question", "next"])?;
    values.ensure_at_most_one(&["repo"])?;
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
        repo: values.repo()?,
        done,
        questions,
        next,
    }))
}

fn parse_skill(args: &[String]) -> Result<Command, String> {
    if args.first().map(String::as_str) != Some("install") {
        return Err("skill requires `install`".to_string());
    }
    let values = Flags::parse(&args[1..])?;
    if !values.positionals.is_empty() {
        return Err("skill install does not accept positional arguments".to_string());
    }
    Ok(Command::SkillInstall {
        path: values.one("path").map(PathBuf::from),
        force: values.present("force"),
    })
}

fn parse_record(args: &[String]) -> Result<Command, String> {
    let kind = args
        .first()
        .cloned()
        .ok_or_else(|| "record requires decision or learning".to_string())?;
    if !matches!(kind.as_str(), "decision" | "learning") {
        return Err("record kind must be decision or learning".to_string());
    }
    let values = Flags::parse(&args[1..])?;
    values.ensure_only(&[
        "repo",
        "title",
        "note",
        "file",
        "topic",
        "evidence",
        "related",
        "supersedes",
    ])?;
    values.ensure_at_most_one(&["repo", "title", "note"])?;
    let title = values.required("title")?;
    let note = values.required("note")?;
    if kind != "decision" && values.present("supersedes") {
        return Err(format!(
            "remove --supersedes: it is only valid for decisions, not {kind} records"
        ));
    }
    Ok(Command::Record(RecordArgs {
        kind,
        repo: values.repo()?,
        title,
        note,
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
                if matches!(name, "changed" | "force") {
                    parsed.values.push((name.to_string(), None));
                    index += 1;
                    continue;
                }
                let next = args
                    .get(index + 1)
                    .filter(|candidate| !candidate.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| format!("--{name} requires a value"))?;
                parsed.values.push((name.to_string(), Some(next)));
                index += 2;
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

    fn required(&self, name: &str) -> Result<String, String> {
        self.one(name)
            .ok_or_else(|| format!("--{name} is required"))
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

    fn repo(&self) -> Result<PathBuf, String> {
        self.one("repo")
            .map(PathBuf::from)
            .map(Ok)
            .unwrap_or_else(|| std::env::current_dir().map_err(|error| error.to_string()))
    }
}

pub fn help() -> &'static str {
    r#"Tincan — a plain-Markdown development journal

USAGE
  tincan init [REPOSITORY]
  tincan inspect [REPOSITORY]
  tincan record <decision|learning> --title TEXT --note TEXT [OPTIONS]
  tincan journal [--repo PATH] [--done TEXT ...] [--question TEXT ...] [--next TEXT ...]
  tincan resume [--repo PATH]
  tincan search [--repo PATH] QUERY
  tincan show [--repo PATH] RECORD_ID
  tincan check [--repo PATH] --changed
  tincan skill install [--path SKILLS_DIRECTORY] [--force]

RECORD OPTIONS
  --repo PATH
  --title TEXT
  --note TEXT
  --file REPOSITORY_PATH         repeatable
  --topic TEXT                   repeatable
  --evidence TEXT                repeatable
  --related ID                   repeatable
  --supersedes DECISION_ID       repeatable; decisions only
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repeated_record_fields() {
        let command = parse(
            [
                "record",
                "learning",
                "--title",
                "Paging",
                "--note",
                "Try pages because startup is slow.",
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
        assert_eq!(record.files, vec!["src/a.rs", "src/b.rs"]);
    }

    #[test]
    fn gives_actionable_record_validation_errors() {
        let learning_status = parse(
            [
                "record",
                "learning",
                "--title",
                "Paging",
                "--note",
                "Measure first",
                "--status",
                "accepted",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .unwrap_err();
        assert_eq!(
            learning_status,
            "unknown option --status; run `tincan --help` and retry with a listed option"
        );

        let typo = parse(
            [
                "record",
                "learning",
                "--title",
                "Paging",
                "--note",
                "Measure first",
                "--files",
                "src/a.rs",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .unwrap_err();
        assert!(typo.contains("unknown option --files"));
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
}
