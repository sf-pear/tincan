use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Init {
        repo: PathBuf,
    },
    Review(ReviewArgs),
    ProjectsList,
    ProjectsUnregister {
        target: Option<String>,
    },
    GitInclude {
        repo: PathBuf,
        yes: bool,
    },
    GitExclude {
        repo: PathBuf,
    },
    Record(RecordArgs),
    Journal(JournalArgs),
    Plan {
        repo: PathBuf,
    },
    Remember {
        repo: PathBuf,
        text: String,
    },
    Later {
        repo: PathBuf,
    },
    Resume {
        repo: PathBuf,
    },
    Search {
        repo: PathBuf,
        query: String,
    },
    Show {
        repo: PathBuf,
        id: String,
    },
    Lift {
        repo: PathBuf,
        id: String,
        from: PathBuf,
    },
    Changes {
        repo: PathBuf,
    },
    SkillInstall {
        path: Option<PathBuf>,
        force: bool,
    },
    SkillStatus,
}

#[derive(Debug, PartialEq)]
pub struct JournalArgs {
    pub repo: PathBuf,
    pub done: Vec<String>,
    pub planned: Vec<String>,
    pub questions: Vec<String>,
    pub next: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct ReviewArgs {
    pub repo: PathBuf,
    pub scope: ReviewScope,
    pub all_projects: bool,
    pub output: Option<PathBuf>,
    pub force: bool,
}

#[derive(Debug, PartialEq)]
pub enum ReviewScope {
    Overview,
    AllTime,
    Year(i32),
    Month(i32, u32),
    Quarter(i32, u32),
    Half(i32, u32),
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
        "summary" => Err(
            "summary was replaced by review; run `tincan review` to inspect recorded history"
                .to_string(),
        ),
        "review" => parse_review(&args[1..]),
        "projects" => parse_projects(&args[1..]),
        "git" => parse_git(&args[1..]),
        "decide" => parse_record("decision", &args[1..]),
        "learn" => parse_record("learning", &args[1..]),
        "journal" => parse_journal(&args[1..]),
        "remember" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if values.positionals.len() != 1 {
                return Err("remember requires exactly one item".to_string());
            }
            Ok(Command::Remember {
                repo: values.directory()?,
                text: values.positionals[0].clone(),
            })
        }
        "later" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if !values.positionals.is_empty() {
                return Err("later does not accept positional arguments".to_string());
            }
            Ok(Command::Later {
                repo: values.directory()?,
            })
        }
        "plan" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if !values.positionals.is_empty() {
                return Err("plan does not accept positional arguments".to_string());
            }
            Ok(Command::Plan {
                repo: values.directory()?,
            })
        }
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
        "lift" => parse_lift(&args[1..]),
        "changes" => {
            let values = Flags::parse(&args[1..])?;
            values.ensure_only(&["directory"])?;
            values.ensure_at_most_one(&["directory"])?;
            if !values.positionals.is_empty() {
                return Err("changes does not accept positional arguments".to_string());
            }
            Ok(Command::Changes {
                repo: values.directory()?,
            })
        }
        "skill" => parse_skill(&args[1..]),
        other => Err(format!("unknown command: {other}\n\n{}", help())),
    }
}

fn parse_projects(args: &[String]) -> Result<Command, String> {
    match args {
        [] => Ok(Command::ProjectsList),
        [action, target] if action == "unregister" => Ok(Command::ProjectsUnregister {
            target: Some(target.clone()),
        }),
        [action] if action == "unregister" => {
            Err("projects unregister requires PATH or WORKSPACE_ID".to_string())
        }
        _ => Err("projects accepts no arguments or `unregister PATH|WORKSPACE_ID`".to_string()),
    }
}

fn parse_git(args: &[String]) -> Result<Command, String> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err(
            "git requires an action; use `tincan git include` or `tincan git exclude`".to_string(),
        );
    };
    if !matches!(action, "include" | "exclude") {
        return Err(format!(
            "unknown git action: {action}; use `tincan git include` or `tincan git exclude`"
        ));
    }
    let values = Flags::parse(&args[1..])?;
    let allowed = if action == "include" {
        &["directory", "yes"][..]
    } else {
        &["directory"][..]
    };
    values.ensure_only(allowed)?;
    values.ensure_at_most_one(allowed)?;
    if !values.positionals.is_empty() {
        return Err(format!("git {action} does not accept positional arguments"));
    }
    if action == "include" {
        Ok(Command::GitInclude {
            repo: values.directory()?,
            yes: values.present("yes"),
        })
    } else {
        Ok(Command::GitExclude {
            repo: values.directory()?,
        })
    }
}

fn parse_lift(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&["directory", "from"])?;
    values.ensure_at_most_one(&["directory", "from"])?;
    if values.positionals.len() != 1 {
        return Err("lift requires exactly one project learning ID".to_string());
    }
    let id = values.positionals[0].clone();
    if Uuid::parse_str(&id).is_err() {
        return Err(format!("lift requires a valid UUID: {id}"));
    }
    let from = values.one("from").map(PathBuf::from).ok_or_else(|| {
        "lift requires --from <MARKDOWN_FILE>; use --from - for stdin".to_string()
    })?;
    Ok(Command::Lift {
        repo: values.directory()?,
        id,
        from,
    })
}

fn parse_journal(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&["directory", "done", "planned", "question", "next"])?;
    values.ensure_at_most_one(&["directory"])?;
    if !values.positionals.is_empty() {
        return Err("journal does not accept positional arguments".to_string());
    }
    let done = values.many("done");
    let planned = values.many("planned");
    let questions = values.many("question");
    let next = values.many("next");
    if done.is_empty() && planned.is_empty() && questions.is_empty() && next.is_empty() {
        return Err("journal requires at least one journal bullet".to_string());
    }
    Ok(Command::Journal(JournalArgs {
        repo: values.directory()?,
        done,
        planned,
        questions,
        next,
    }))
}

fn parse_review(args: &[String]) -> Result<Command, String> {
    let values = Flags::parse(args)?;
    values.ensure_only(&[
        "directory",
        "year",
        "month",
        "quarter",
        "half",
        "all-time",
        "all-projects",
        "output",
        "force",
    ])?;
    values.ensure_at_most_one(&[
        "directory",
        "year",
        "month",
        "quarter",
        "half",
        "all-time",
        "all-projects",
        "output",
        "force",
    ])?;
    if !values.positionals.is_empty() {
        return Err("review does not accept positional scopes; use --year, --month, --quarter, --half, or --all-time".to_string());
    }
    if values.present("force") && !values.present("output") {
        return Err("--force requires --output <FILE>".to_string());
    }
    let time_flags = ["year", "month", "quarter", "half", "all-time"]
        .into_iter()
        .filter(|name| values.present(name))
        .collect::<Vec<_>>();
    if time_flags.len() > 1 {
        return Err(format!(
            "review time options are mutually exclusive: {}",
            time_flags
                .iter()
                .map(|name| format!("--{name}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let scope = match time_flags.first().copied() {
        None => ReviewScope::Overview,
        Some("all-time") => ReviewScope::AllTime,
        Some("year") => ReviewScope::Year(parse_year(&values.one("year").unwrap(), "--year")?),
        Some("month") => {
            let (year, month) = parse_calendar_period(&values.one("month").unwrap(), "month", 12)?;
            ReviewScope::Month(year, month)
        }
        Some("quarter") => {
            let (year, quarter) =
                parse_calendar_period(&values.one("quarter").unwrap(), "quarter", 4)?;
            ReviewScope::Quarter(year, quarter)
        }
        Some("half") => {
            let (year, half) = parse_calendar_period(&values.one("half").unwrap(), "half", 2)?;
            ReviewScope::Half(year, half)
        }
        Some(_) => unreachable!(),
    };
    Ok(Command::Review(ReviewArgs {
        repo: values.directory()?,
        scope,
        all_projects: values.present("all-projects"),
        output: values.one("output").map(PathBuf::from),
        force: values.present("force"),
    }))
}

fn parse_year(value: &str, option: &str) -> Result<i32, String> {
    if value.len() != 4 || !value.chars().all(|character| character.is_ascii_digit()) {
        return Err(format!("{option} requires a four-digit year: {value}"));
    }
    value
        .parse::<i32>()
        .ok()
        .filter(|year| (1..=9999).contains(year))
        .ok_or_else(|| format!("{option} requires a year from 0001 to 9999: {value}"))
}

fn parse_calendar_period(value: &str, period: &str, maximum: u32) -> Result<(i32, u32), String> {
    let (year, number) = value.split_once('-').ok_or_else(|| {
        format!(
            "--{period} requires YYYY-{}: {value}",
            period_format(period)
        )
    })?;
    let year = parse_year(year, &format!("--{period}"))?;
    let number = match period {
        "month" if number.len() == 2 => number,
        "quarter" => number.strip_prefix('Q').unwrap_or(""),
        "half" => number.strip_prefix('H').unwrap_or(""),
        _ => "",
    }
    .parse::<u32>()
    .map_err(|_| {
        format!(
            "--{period} requires YYYY-{}: {value}",
            period_format(period)
        )
    })?;
    if !(1..=maximum).contains(&number) {
        return Err(format!(
            "--{period} requires a value from 1 to {maximum}: {value}"
        ));
    }
    Ok((year, number))
}

fn period_format(period: &str) -> &'static str {
    match period {
        "month" => "MM",
        "quarter" => "QN",
        "half" => "HN",
        _ => unreachable!(),
    }
}

fn parse_skill(args: &[String]) -> Result<Command, String> {
    match args.first().map(String::as_str) {
        Some("install") => {
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
        Some("status") if args.len() == 1 => Ok(Command::SkillStatus),
        Some("status") => Err("skill status does not accept arguments".to_string()),
        _ => Err("skill requires `install` or `status`".to_string()),
    }
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
                if matches!(
                    name,
                    "changed" | "force" | "verbose" | "all-time" | "all-projects" | "yes"
                ) {
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

Tincan keeps a living plan, daily journal, durable decisions, and learnings
under a workspace's .tincan/ directory. Markdown remains the source of truth.

USAGE
  tincan <COMMAND> [OPTIONS]

COMMANDS
  init [DIRECTORY]              Initialize private .tincan/ storage
  review [OPTIONS]              Inspect history or select retrospective context
  projects                      List registered workspaces and their status
  projects unregister TARGET    Forget one registration without changing projects
  git include [OPTIONS]          Make .tincan visible to Git without tracking it
  git exclude [OPTIONS]          Exclude .tincan from Git locally
  plan [-d|--directory PATH]    Print the living project plan
  remember [-d|--directory PATH] TEXT
                                Keep one informal item for later
  later [-d|--directory PATH]   Print the informal later shelf
  journal [OPTIONS]             Update today's concise work record
  resume [-d|--directory PATH]  Print the plan, later shelf, and latest journal
  decide STATEMENT [OPTIONS]    Create an accepted decision record
  learn STATEMENT [OPTIONS]     Create an evidence-supported learning record
  search [-d|--directory PATH] QUERY
                                List matching records and their IDs
  show [-d|--directory PATH] RECORD_ID
                                Print one record; use an ID returned by search
  lift [-d|--directory PATH] RECORD_ID --from MARKDOWN_FILE
                                Save an approved generalized global learning
  changes [-d|--directory PATH] Show memory related to Git-changed paths
  skill install [OPTIONS]       Install the bundled Agent Skill
  skill status                  Check detected Agent Skill installations
  help, --help                  Print this help
  version, --version            Print the installed Tincan version

WORKSPACES AND FILES
  Commands use the nearest parent directory containing .tincan/config.toml.
  A workspace may contain zero, one, or several Git repositories. -d is the
  short form of --directory. Tincan writes only under .tincan/ and keeps it out
  of normal Git tracking when the workspace is inside a repository.
  Global learnings live under ~/.tincan/global/learnings. Set TINCAN_HOME to
  relocate the personal .tincan directory. Search and show include global
  learnings, including when run outside a project workspace. Initialized
  workspaces are registered by stable ID for optional cross-project reviews.

RECORD IDS
  Decisions and learnings receive stable UUID v7 IDs such as
  019c4ea8-7e42-7b31-a211-8df9357d747c. `decide` and `learn` print the new ID;
  `search` prints matching project and global IDs. `show` is a convenience for
  agents and scripts. `lift` reads an approved, generalized Markdown body from
  --from (or stdin with --from -), gives it a new ID, and preserves its source
  learning as provenance.

EXAMPLES
  tincan init .
  tincan plan
  tincan remember "Explore a shorter review default"
  tincan later
  tincan review
  tincan review --quarter 2025-Q3
  tincan review --year 2025 --all-projects
  tincan projects
  tincan git include
  tincan git exclude
  tincan review --all-time --output review.md
  tincan journal --done "Implemented deterministic path matching"
  tincan decide "Keep Markdown canonical" --topic storage
  tincan learn "Paging did not reduce rendering work" --evidence "Release trace"
  tincan search "markdown"
  tincan show 019c4ea8-7e42-7b31-a211-8df9357d747c
  tincan lift 019c4ea8-7e42-7b31-a211-8df9357d747c --from global-learning.md

JOURNAL OPTIONS
  tincan journal [-d|--directory PATH] [--done TEXT ...]
                 [--planned TEXT ...] [--question TEXT ...] [--next TEXT ...]

  At least one journal bullet is required. Each bullet option is repeatable.

REVIEW
  tincan review [-d|--directory PATH]
  tincan review --year YEAR [--output FILE] [--force]
  tincan review --month YYYY-MM [--output FILE] [--force]
  tincan review --quarter YYYY-QN [--output FILE] [--force]
  tincan review --half YYYY-HN [--output FILE] [--force]
  tincan review --all-time [--output FILE] [--force]

  With no time option, show recorded date coverage and yearly counts. Time
  options are mutually exclusive. The current project is the default; add
  --all-projects to read every registered project. Output files are created
  without overwriting unless --force is provided.

PROJECTS
  tincan projects
  tincan projects unregister PATH|WORKSPACE_ID

  List registrations and whether each path is available. Unregister one by path
  or ID. Project .tincan directories are untouched.

GIT INCLUDE
  tincan git include [-d|--directory PATH] [--yes]

  Remove only Tincan's exact rule from the repository's local Git exclude file.
  This makes .tincan visible to Git but does not add or commit any files. The
  command asks for confirmation unless --yes is provided. Other ignore rules
  may still keep .tincan hidden.

GIT EXCLUDE
  tincan git exclude [-d|--directory PATH]

  Add Tincan's exact rule to the repository's local Git exclude file. This
  restores the private default without changing any tracked files.

SKILL INSTALL
  tincan skill install [--path SKILLS_DIRECTORY] [--force]

  Without --path, detect user-wide Agent Skills harnesses and select all by
  default. Each choice says whether Tincan will be installed or updated;
  already-current installations are omitted. Use arrow keys to move, Space to
  select or unselect, A to toggle all or none, Enter to continue, and Escape to
  cancel. Every installation requires answering a final Y/n confirmation;
  pressing Enter accepts. Use --path for an explicit, non-interactive or
  project-local destination.

SKILL STATUS
  tincan skill status

  Report overall Agent Skill health and group detected installations by update
  state. This command is read-only and works without an interactive terminal.

RECORD OPTIONS
  tincan decide STATEMENT [OPTIONS]
  tincan learn STATEMENT [OPTIONS]

  -d, --directory PATH
  --file WORKSPACE_PATH          repeatable
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
    fn parses_read_only_skill_status() {
        assert_eq!(
            parse(["skill", "status"].map(str::to_string).to_vec()).unwrap(),
            Command::SkillStatus
        );
        assert!(
            parse(["skill", "status", "extra"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("does not accept arguments")
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
        assert!(journal.planned.is_empty());
        assert_eq!(journal.questions, vec!["Should topics be normalized?"]);
        assert_eq!(journal.next, vec!["Test on another repository"]);
    }

    #[test]
    fn parses_later_shelf_commands() {
        assert_eq!(
            parse(
                ["remember", "Explore a shorter review default"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Remember {
                repo: std::env::current_dir().unwrap(),
                text: "Explore a shorter review default".to_string(),
            }
        );
        assert_eq!(
            parse(["later", "-d", "workspace"].map(str::to_string).to_vec()).unwrap(),
            Command::Later {
                repo: PathBuf::from("workspace"),
            }
        );
        assert!(parse(vec!["remember".to_string()]).is_err());
        assert!(parse(["later", "extra"].map(str::to_string).to_vec()).is_err());
    }

    #[test]
    fn rejects_decisions_and_learnings_as_journal_duplicates() {
        for option in ["--decision", "--learning"] {
            let error = parse(
                ["journal", option, "Keep Markdown canonical"]
                    .map(str::to_string)
                    .to_vec(),
            )
            .unwrap_err();
            assert!(error.contains(&format!("unknown option {option}")));
        }
    }

    #[test]
    fn parses_review_scopes_and_rejects_removed_summary() {
        assert!(matches!(
            parse(vec!["review".to_string()]).unwrap(),
            Command::Review(ReviewArgs {
                scope: ReviewScope::Overview,
                all_projects: false,
                ..
            })
        ));
        assert!(matches!(
            parse(
                ["review", "--quarter", "2025-Q3", "--all-projects"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Review(ReviewArgs {
                scope: ReviewScope::Quarter(2025, 3),
                all_projects: true,
                ..
            })
        ));
        assert!(matches!(
            parse(
                ["review", "--all-time", "--output", "review.md", "--force"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Review(ReviewArgs {
                scope: ReviewScope::AllTime,
                output: Some(output),
                force: true,
                ..
            }) if output == std::path::Path::new("review.md")
        ));
        assert!(
            parse(["summary"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("replaced by review")
        );
        assert!(parse(vec!["inspect".to_string()]).is_err());
    }

    #[test]
    fn parses_project_registry_commands() {
        assert_eq!(
            parse(vec!["projects".into()]).unwrap(),
            Command::ProjectsList
        );
        let id = "019fd6d9-1ff8-7082-9f86-2b7d89712a57";
        assert_eq!(
            parse(["projects", "unregister", id].map(str::to_string).to_vec()).unwrap(),
            Command::ProjectsUnregister {
                target: Some(id.into())
            }
        );
        assert!(
            parse(["projects", "unregister"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("requires PATH or WORKSPACE_ID")
        );
    }

    #[test]
    fn parses_git_include_with_explicit_confirmation() {
        assert!(matches!(
            parse(
                ["git", "include", "--directory", "project", "--yes"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::GitInclude { repo, yes: true }
                if repo == std::path::Path::new("project")
        ));
        assert!(matches!(
            parse(["git", "exclude"].map(str::to_string).to_vec()).unwrap(),
            Command::GitExclude { .. }
        ));
        assert!(
            parse(["git", "remove"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("use `tincan git include`")
        );
    }

    #[test]
    fn validates_review_periods_and_option_combinations() {
        for args in [
            vec!["review", "--month", "2025-13"],
            vec!["review", "--quarter", "2025-Q0"],
            vec!["review", "--half", "2025-H3"],
            vec!["review", "--quarter", "2025-3"],
            vec!["review", "--all-time", "--year", "2025"],
            vec!["review", "--force"],
            vec!["review", "all"],
        ] {
            assert!(parse(args.into_iter().map(str::to_string).collect()).is_err());
        }
        assert!(matches!(
            parse(
                ["review", "--month", "2025-08"]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Review(ReviewArgs {
                scope: ReviewScope::Month(2025, 8),
                ..
            })
        ));
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
    fn parses_lift_with_a_project_learning_id() {
        let id = "019c4ea8-7e42-7b31-a211-8df9357d747c";
        assert_eq!(
            parse(
                ["lift", id, "--directory", "project", "--from", "global.md",]
                    .map(str::to_string)
                    .to_vec()
            )
            .unwrap(),
            Command::Lift {
                repo: PathBuf::from("project"),
                id: id.to_string(),
                from: PathBuf::from("global.md"),
            }
        );
        assert!(
            parse(["lift", "not-an-id"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("valid UUID")
        );
        assert!(
            parse(["lift", id].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("requires --from")
        );
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
            parse(vec!["changes".to_string()]).unwrap(),
            Command::Changes {
                repo: std::env::current_dir().unwrap(),
            }
        );
        assert!(
            parse(["changes", "--changed"].map(str::to_string).to_vec())
                .unwrap_err()
                .contains("unknown option --changed")
        );
    }

    #[test]
    fn help_explains_record_ids_and_direct_markdown_access() {
        assert!(help().contains("use an ID returned by search"));
        assert!(help().contains("nearest parent directory"));
        assert!(help().contains("UUID v7 IDs"));
        assert!(help().contains("tincan decide STATEMENT"));
        assert!(help().contains("tincan lift"));
        assert!(!help().contains("record decision"));
    }
}
