use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Init {
        repo: PathBuf,
    },
    Inspect {
        repo: PathBuf,
    },
    Record(RecordArgs),
    Capture(CaptureArgs),
    Handoff(HandoffArgs),
    FieldNote {
        repo: PathBuf,
        source_id: String,
        title: Option<String>,
    },
    Search {
        repo: PathBuf,
        query: String,
    },
    CheckChanged {
        repo: PathBuf,
    },
    SkillInstall {
        path: Option<PathBuf>,
        force: bool,
    },
}

#[derive(Debug, PartialEq)]
pub struct RecordArgs {
    pub kind: String,
    pub repo: PathBuf,
    pub title: String,
    pub summary: String,
    pub result: Option<String>,
    pub status: Option<String>,
    pub affects: Vec<String>,
    pub topics: Vec<String>,
    pub evidence: Vec<String>,
    pub related: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct CaptureArgs {
    pub repo: PathBuf,
    pub title: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct HandoffArgs {
    pub repo: PathBuf,
    pub title: Option<String>,
    pub next: Vec<String>,
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
        "capture" => {
            let values = Flags::parse(&args[1..])?;
            Ok(Command::Capture(CaptureArgs {
                repo: values.repo()?,
                title: values.one("title"),
            }))
        }
        "handoff" => {
            let values = Flags::parse(&args[1..])?;
            Ok(Command::Handoff(HandoffArgs {
                repo: values.repo()?,
                title: values.one("title"),
                next: values.many("next"),
            }))
        }
        "field-note" => {
            let values = Flags::parse(&args[1..])?;
            Ok(Command::FieldNote {
                repo: values.repo()?,
                source_id: values.required("from")?,
                title: values.one("title"),
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
        .ok_or_else(|| "record requires attempt, decision, or learning".to_string())?;
    if !matches!(kind.as_str(), "attempt" | "decision" | "learning") {
        return Err("record kind must be attempt, decision, or learning".to_string());
    }
    let values = Flags::parse(&args[1..])?;
    let title = values.required("title")?;
    let summary = values.required("summary")?;
    Ok(Command::Record(RecordArgs {
        kind,
        repo: values.repo()?,
        title,
        summary,
        result: values.one("result"),
        status: values.one("status"),
        affects: values.many("affects"),
        topics: values.many("topic"),
        evidence: values.many("evidence"),
        related: values.many("related"),
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
  tincan record <attempt|decision|learning> --title TEXT --summary TEXT [OPTIONS]
  tincan capture [--repo PATH] [--title TEXT]
  tincan handoff [--repo PATH] [--title TEXT] [--next TEXT ...]
  tincan field-note [--repo PATH] --from RECORD_ID [--title TEXT]
  tincan search [--repo PATH] QUERY
  tincan check [--repo PATH] --changed
  tincan skill install [--path SKILLS_DIRECTORY] [--force]

RECORD OPTIONS
  --repo PATH
  --title TEXT
  --summary TEXT
  --result TEXT
  --status TEXT
  --affects PATH_OR_MODULE       repeatable
  --topic TEXT                   repeatable
  --evidence TEXT                repeatable
  --related ID                   repeatable
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
                "attempt",
                "--title",
                "Paging",
                "--summary",
                "Try pages",
                "--affects",
                "src/a.rs",
                "--affects",
                "gallery",
            ]
            .map(str::to_string)
            .to_vec(),
        )
        .unwrap();

        let Command::Record(record) = command else {
            panic!("expected record command");
        };
        assert_eq!(record.affects, vec!["src/a.rs", "gallery"]);
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
}
