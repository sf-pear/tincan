use crate::branding;
use crate::cli::{self, Command, JournalArgs, RecordArgs};
use crate::git;
use crate::model::{DecisionStatus, Kind, Record};
use crate::skill::{self, InstallOutcome};
use crate::store;
use crate::util::display_path;
use crate::workspace;
use chrono::{Local, SecondsFormat};
use uuid::Uuid;

pub fn run(command: Result<Command, String>) -> Result<(), String> {
    let command = command?;
    let notify_about_skill_update =
        !matches!(command, Command::SkillInstall { .. } | Command::SkillStatus);
    let result = match command {
        Command::Help => {
            branding::print();
            print!("{}", cli::help());
            Ok(())
        }
        Command::Version => {
            println!("tincan {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Init { repo } => init(repo),
        Command::Summary { repo, verbose } => summary(repo, verbose),
        Command::Record(args) => record(args),
        Command::Journal(args) => journal(args),
        Command::Plan { repo } => plan(repo),
        Command::Resume { repo } => resume(repo),
        Command::Search { repo, query } => search(repo, &query),
        Command::Show { repo, id } => show(repo, &id),
        Command::Changes { repo } => changes(repo),
        Command::SkillInstall { path, force } => install_skill(path, force),
        Command::SkillStatus => skill_status(),
    };
    if result.is_ok() && notify_about_skill_update {
        skill::notify_if_update_available();
    }
    result
}

fn skill_status() -> Result<(), String> {
    for line in skill_status_lines(skill::installation_statuses()) {
        println!("{line}");
    }
    Ok(())
}

fn skill_status_lines(statuses: Vec<skill::InstallationStatus>) -> Vec<String> {
    if statuses.is_empty() {
        return vec!["No supported Agent Skills destinations detected.".into()];
    }
    let current = statuses
        .iter()
        .filter(|status| status.state == skill::InstallationState::Current)
        .collect::<Vec<_>>();
    let outdated = statuses
        .iter()
        .filter(|status| status.state == skill::InstallationState::UpdateAvailable)
        .collect::<Vec<_>>();
    if current.is_empty() && outdated.is_empty() {
        let mut lines = vec!["Agent Skill is not installed.".into()];
        append_skill_locations(&mut lines, "Available destinations:", statuses.iter());
        lines.push("Run `tincan skill install` to install it.".into());
        return lines;
    }
    if outdated.is_empty() {
        let mut lines = vec!["Agent Skill is up to date everywhere.".into()];
        append_skill_locations(&mut lines, "Installed in:", current.into_iter());
        return lines;
    }
    let mut lines = vec!["Agent Skill update available.".into()];
    append_skill_locations(&mut lines, "Needs update:", outdated.into_iter());
    if !current.is_empty() {
        append_skill_locations(&mut lines, "Already up to date:", current.into_iter());
    }
    lines.push("Run `tincan skill install` to update.".into());
    lines
}

fn append_skill_locations<'a>(
    lines: &mut Vec<String>,
    heading: &str,
    statuses: impl Iterator<Item = &'a skill::InstallationStatus>,
) {
    lines.push(heading.into());
    lines.extend(statuses.map(|status| {
        format!(
            "- {} ({})",
            status.root.name,
            skill::display_user_path(&status.root.path.join("tincan"))
        )
    }));
}

fn install_skill(path: Option<std::path::PathBuf>, force: bool) -> Result<(), String> {
    branding::print();
    let overwrite = overwrite_existing_skill(path.as_deref(), force);
    let roots = match path {
        Some(path) => vec![path],
        None => {
            let detected = skill::detect_roots();
            if detected.is_empty() {
                return Err(
                    "no supported Agent Skills destination was detected; pass `--path <skills-directory>`"
                        .to_string(),
                );
            }
            let Some(selected) = skill::choose_interactively(&detected)? else {
                println!("Skill installation cancelled.");
                return Ok(());
            };
            selected
        }
    };

    let outcomes = skill::install_many(&roots, overwrite)?;
    let mut installed = false;
    for outcome in outcomes {
        match outcome {
            InstallOutcome::Installed(path) => {
                installed = true;
                println!(
                    "Installed Tincan skill at {}",
                    skill::display_user_path(&path)
                );
            }
            InstallOutcome::AlreadyCurrent(path) => {
                println!(
                    "Tincan skill is already current at {}",
                    skill::display_user_path(&path)
                );
            }
        }
    }
    if installed {
        println!("Restart or reload the agent harness to discover it.");
    }
    Ok(())
}

fn overwrite_existing_skill(path: Option<&std::path::Path>, force: bool) -> bool {
    path.is_none() || force
}

fn init(path: std::path::PathBuf) -> Result<(), String> {
    let root = workspace::target(&path)?;
    let excluded = git::protect_workspace(&root)?;
    let tincan = store::initialize(&root)?;
    branding::print();
    println!("Initialized Tincan at {}", display_path(&tincan));
    match excluded {
        Some(true) => println!("Kept .tincan private through Git's local exclude file."),
        Some(false) => println!(".tincan is already excluded from Git locally."),
        None => {
            println!("This workspace is outside Git; nested repositories cannot track .tincan.")
        }
    }
    Ok(())
}

fn summary(path: std::path::PathBuf, verbose: bool) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let documents = store::scan(&root)?;
    let groups = [
        ("Decisions", "decision"),
        ("Learnings", "learning"),
        ("Journals", "journal"),
    ];
    for (label, kind) in groups {
        print_summary_count(label, kind, &documents);
    }
    if verbose {
        for (label, kind) in groups {
            print_summary_details(label, kind, &root, &documents);
        }
    }
    Ok(())
}

fn print_summary_count(label: &str, kind: &str, documents: &[store::Document]) {
    let count = documents
        .iter()
        .filter(|document| document.kind == kind)
        .count();
    let padded_label = format!("{label:<9}");
    println!("{} {count}", branding::section(&padded_label));
}

fn print_summary_details(
    label: &str,
    kind: &str,
    root: &std::path::Path,
    documents: &[store::Document],
) {
    let matching: Vec<_> = documents
        .iter()
        .filter(|document| document.kind == kind)
        .collect();
    if matching.is_empty() {
        return;
    }
    println!();
    println!("{}", branding::section(label));
    for document in matching {
        let relative = document.path.strip_prefix(root).unwrap_or(&document.path);
        println!(
            "  {}  {}",
            branding::heading(&document.heading),
            branding::path(&display_path(relative))
        );
    }
}

fn record(args: RecordArgs) -> Result<(), String> {
    if args.kind != "decision" && !args.supersedes.is_empty() {
        return Err("--supersedes can only be used with a decision".to_string());
    }
    let root = workspace::find(&args.repo)?;
    let kind = Kind::parse(&args.kind)?;
    let id = Uuid::now_v7().to_string();
    let created_at = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let status = match kind {
        Kind::Decision => Some(DecisionStatus::Active),
        Kind::Learning => None,
        Kind::Journal => unreachable!("journal entries use the journal command"),
    };
    let superseded = store::active_decisions(&root, &args.supersedes)?;
    let record = Record {
        id,
        kind,
        created_at,
        statement: args.statement,
        status,
        files: args.files,
        topics: args.topics,
        evidence: args.evidence,
        related: args.related,
        supersedes: args.supersedes,
        branch: git::branch(&args.repo)?,
    };
    let path = store::write(&root, kind, &record.id, &record.render())?;
    if let Err(error) = store::mark_superseded(&superseded, &record.id) {
        return match std::fs::remove_file(&path) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(format!(
                "{error}; also could not remove incomplete replacement {}: {cleanup_error}",
                path.display()
            )),
        };
    }
    println!("Created {}: {}", kind.as_str(), display_path(&path));
    println!("Record ID: {}", record.id);
    println!("Add detailed context directly to the Markdown body when useful.");
    if !superseded.is_empty() {
        println!("Superseded {} earlier decision(s).", superseded.len());
    }
    Ok(())
}

fn journal(args: JournalArgs) -> Result<(), String> {
    let root = workspace::find(&args.repo)?;
    store::require(&root)?;
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let created_at = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    let sections = store::JournalSections {
        done: &args.done,
        planned: &args.planned,
        questions: &args.questions,
        next: &args.next,
    };
    let update = store::update_journal(&root, &date, &created_at, sections)?;
    println!("Updated journal: {}", display_path(&update.path));
    if update.added == 0 {
        println!("No new bullets; exact duplicates were already present.");
    } else {
        println!("Added {} bullet(s).", update.added);
    }
    Ok(())
}

fn resume(path: std::path::PathBuf) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let Some((journal_path, content)) = store::latest_journal(&root)? else {
        println!("No journal entries yet.");
        println!("Use `tincan journal --done <text>` as meaningful work develops.");
        return Ok(());
    };
    println!("Latest journal: {}\n", display_path(&journal_path));
    print!("{content}");
    Ok(())
}

fn plan(path: std::path::PathBuf) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let (plan_path, content) = store::read_plan(&root)?;
    println!("Plan: {}\n", display_path(&plan_path));
    print!("{content}");
    Ok(())
}

fn search(path: std::path::PathBuf, query: &str) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let query = query.to_lowercase();
    let mut matches: Vec<_> = store::scan(&root)?
        .into_iter()
        .filter_map(|document| search_rank(&document, &query).map(|rank| (rank, document)))
        .collect();
    matches.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.path.cmp(&right.1.path)));
    if matches.is_empty() {
        println!("No Tincan records matched.");
        return Ok(());
    }
    for (_, document) in matches {
        print_document_summary(&document, Some(&query));
    }
    Ok(())
}

fn show(path: std::path::PathBuf, id: &str) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let document = store::scan(&root)?
        .into_iter()
        .find(|document| document.id == id)
        .ok_or_else(|| format!("no Tincan record found with id {id}"))?;
    print!("{}", store::read_document(&document)?);
    Ok(())
}

fn search_rank(document: &store::Document, query: &str) -> Option<u8> {
    if document.id.to_lowercase() == query {
        return Some(0);
    }
    if document.heading.to_lowercase().contains(query) {
        return Some(1);
    }
    if metadata_text(document).to_lowercase().contains(query) {
        return Some(2);
    }
    document.body.to_lowercase().contains(query).then_some(3)
}

fn metadata_text(document: &store::Document) -> String {
    [
        vec![
            document.id.clone(),
            document.kind.clone(),
            document.status.clone().unwrap_or_default(),
        ],
        document.files.clone(),
        document.topics.clone(),
        document.related.clone(),
        document.supersedes.clone(),
        document.superseded_by.clone(),
    ]
    .concat()
    .join("\n")
}

fn print_document_summary(document: &store::Document, query: Option<&str>) {
    let label = document
        .status
        .as_deref()
        .map(|status| format!("{} / {status}", document.kind))
        .unwrap_or_else(|| document.kind.clone());
    println!("{} [{label}]", document.heading);
    println!("  id: {}", document.id);
    if let Some(excerpt) = query.and_then(|query| matching_excerpt(document, query)) {
        println!("  matched: {excerpt}");
    }
    if !document.files.is_empty() {
        println!("  files: {}", document.files.join(", "));
    }
    if !document.topics.is_empty() {
        println!("  topics: {}", document.topics.join(", "));
    }
    if !document.supersedes.is_empty() {
        println!("  supersedes: {}", document.supersedes.join(", "));
    }
    if !document.superseded_by.is_empty() {
        println!("  superseded by: {}", document.superseded_by.join(", "));
    }
    println!("  {}", display_path(&document.path));
}

fn matching_excerpt(document: &store::Document, query: &str) -> Option<String> {
    let query = query.to_lowercase();
    document.body.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty()
            || line.strip_prefix("# ") == Some(document.heading.as_str())
            || !line.to_lowercase().contains(&query)
        {
            return None;
        }
        let cleaned = line
            .trim_start_matches('#')
            .trim_start_matches(['-', '*'])
            .trim();
        let mut excerpt: String = cleaned.chars().take(120).collect();
        if cleaned.chars().count() > 120 {
            excerpt.push('…');
        }
        Some(excerpt)
    })
}

fn changes(path: std::path::PathBuf) -> Result<(), String> {
    let root = workspace::find(&path)?;
    let Some(changed) = git::workspace_changed_files(&root)? else {
        println!("No Git repositories found in this Tincan workspace.");
        return Ok(());
    };
    if changed.is_empty() {
        println!("No changed files.");
        return Ok(());
    }

    let documents = store::scan(&root)?;
    for file in &changed {
        let related: Vec<_> = documents
            .iter()
            .filter(|document| {
                document
                    .files
                    .iter()
                    .any(|affected| paths_overlap(file, affected))
            })
            .collect();
        if related.is_empty() {
            println!(
                "{}  {}",
                branding::section(file),
                branding::path("no records")
            );
            continue;
        }
        for (index, document) in related.into_iter().enumerate() {
            let relative = document.path.strip_prefix(&root).unwrap_or(&document.path);
            let file_label = if index == 0 { file.as_str() } else { "" };
            println!(
                "{file_label}  {}: {}  {}",
                document.kind,
                branding::heading(&document.heading),
                branding::path(&display_path(relative))
            );
        }
    }
    Ok(())
}

fn paths_overlap(changed: &str, affected: &str) -> bool {
    let changed = changed.trim_matches('/').replace('\\', "/");
    let affected = affected.trim_matches('/').replace('\\', "/");
    changed == affected
        || changed.starts_with(&(affected.clone() + "/"))
        || affected.starts_with(&(changed + "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::process::Command as ProcessCommand;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn matches_files_and_directories() {
        assert!(paths_overlap("src/feature/a.rs", "src/feature"));
        assert!(paths_overlap("src/feature", "src/feature/a.rs"));
        assert!(!paths_overlap("src/a.rs", "src/b.rs"));
    }

    #[test]
    fn confirmed_interactive_skill_install_updates_existing_destinations() {
        assert!(overwrite_existing_skill(None, false));
        assert!(!overwrite_existing_skill(
            Some(std::path::Path::new("custom-skills")),
            false
        ));
        assert!(overwrite_existing_skill(
            Some(std::path::Path::new("custom-skills")),
            true
        ));
    }

    #[test]
    fn skill_status_groups_outdated_and_current_installations() {
        let statuses = vec![
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Current".into(),
                    path: std::path::PathBuf::from("current"),
                },
                state: skill::InstallationState::Current,
            },
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Outdated".into(),
                    path: std::path::PathBuf::from("outdated"),
                },
                state: skill::InstallationState::UpdateAvailable,
            },
            skill::InstallationStatus {
                root: skill::SkillRoot {
                    name: "Missing".into(),
                    path: std::path::PathBuf::from("missing"),
                },
                state: skill::InstallationState::NotInstalled,
            },
        ];

        let lines = skill_status_lines(statuses);
        assert_eq!(lines[0], "Agent Skill update available.");
        assert_eq!(lines[1], "Needs update:");
        assert!(lines[2].contains("Outdated"));
        assert_eq!(lines[3], "Already up to date:");
        assert!(lines[4].contains("Current"));
        assert!(lines.iter().all(|line| !line.contains("Missing")));
    }

    #[test]
    fn skill_status_leads_with_health_then_lists_installed_locations() {
        let statuses = vec![skill::InstallationStatus {
            root: skill::SkillRoot {
                name: "Current".into(),
                path: std::path::PathBuf::from("current"),
            },
            state: skill::InstallationState::Current,
        }];
        let lines = skill_status_lines(statuses);
        assert_eq!(lines[0], "Agent Skill is up to date everywhere.");
        assert_eq!(lines[1], "Installed in:");
        assert!(lines[2].contains("Current"));
    }

    #[test]
    fn creates_uuid_record_that_remains_searchable_after_body_edits() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-uuid-record-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        assert!(
            ProcessCommand::new("git")
                .args(["init", "--quiet"])
                .current_dir(&repo)
                .status()
                .unwrap()
                .success()
        );
        store::initialize(&repo).unwrap();

        record(RecordArgs {
            kind: "learning".to_string(),
            repo: repo.clone(),
            statement: "Paging did not reduce rendering work".to_string(),
            files: vec!["src/gallery.rs".to_string()],
            topics: vec!["performance".to_string()],
            evidence: vec!["Release trace".to_string()],
            related: Vec::new(),
            supersedes: Vec::new(),
        })
        .unwrap();

        let document = store::scan(&repo).unwrap().remove(0);
        assert!(Uuid::parse_str(&document.id).is_ok());
        assert_eq!(
            document.path.file_stem().and_then(|value| value.to_str()),
            Some(document.id.as_str())
        );
        writeln!(
            OpenOptions::new()
                .append(true)
                .open(&document.path)
                .unwrap(),
            "The renderer remained the measured bottleneck."
        )
        .unwrap();

        let edited = store::scan(&repo).unwrap().remove(0);
        assert_eq!(search_rank(&edited, "renderer"), Some(3));
        assert_eq!(
            matching_excerpt(&edited, "renderer").as_deref(),
            Some("The renderer remained the measured bottleneck.")
        );
        fs::remove_dir_all(repo).unwrap();
    }
}
