use crate::branding;
use crate::cli::{self, Command, JournalArgs, RecordArgs};
use crate::git;
use crate::model::{DecisionStatus, Kind, Record};
use crate::skill::{self, InstallOutcome};
use crate::store;
use crate::util::display_path;
use chrono::{Local, SecondsFormat};
use std::collections::BTreeMap;
use uuid::Uuid;

pub fn run(command: Result<Command, String>) -> Result<(), String> {
    match command? {
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
        Command::Summary { repo } => summary(repo),
        Command::Record(args) => record(args),
        Command::Journal(args) => journal(args),
        Command::Resume { repo } => resume(repo),
        Command::Search { repo, query } => search(repo, &query),
        Command::Show { repo, id } => show(repo, &id),
        Command::Check { repo } => check(repo),
        Command::SkillInstall { path, force } => install_skill(path, force),
    }
}

fn install_skill(path: Option<std::path::PathBuf>, force: bool) -> Result<(), String> {
    branding::print();
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

    let outcomes = skill::install_many(&roots, force)?;
    let mut installed = false;
    for outcome in outcomes {
        match outcome {
            InstallOutcome::Installed(path) => {
                installed = true;
                println!("Installed Tincan skill at {}", display_path(&path));
            }
            InstallOutcome::AlreadyCurrent(path) => {
                println!("Tincan skill is already current at {}", display_path(&path));
            }
        }
    }
    if installed {
        println!("Restart or reload the agent harness to discover it.");
    }
    Ok(())
}

fn init(path: std::path::PathBuf) -> Result<(), String> {
    let root = git::repository_root(&path)?;
    git::require_tincan_untracked(&root)?;
    let exclude = git::exclude_path(&root)?;
    let excluded = store::ensure_git_excluded(&exclude)?;
    git::verify_tincan_ignored(&root)?;
    let tincan = store::initialize(&root)?;
    branding::print();
    println!("Initialized Tincan at {}", display_path(&tincan));
    if excluded {
        println!("Kept .tincan private through Git's local exclude file.");
    } else {
        println!(".tincan is already excluded from Git locally.");
    }
    Ok(())
}

fn summary(path: std::path::PathBuf) -> Result<(), String> {
    let snapshot = git::snapshot(&path)?;
    let documents = store::scan(&snapshot.root)?;
    let mut counts = BTreeMap::new();
    for document in &documents {
        *counts.entry(document.kind.as_str()).or_insert(0usize) += 1;
    }

    println!("Repository: {}", display_path(&snapshot.root));
    println!(
        "Branch: {}",
        if snapshot.branch.is_empty() {
            "(detached)"
        } else {
            &snapshot.branch
        }
    );
    println!("Changed files: {}", snapshot.changed_files.len());
    println!("Records: {}", documents.len());
    for (kind, count) in counts {
        println!("  {kind}: {count}");
    }
    Ok(())
}

fn record(args: RecordArgs) -> Result<(), String> {
    if args.kind != "decision" && !args.supersedes.is_empty() {
        return Err("--supersedes can only be used with a decision".to_string());
    }
    let snapshot = git::snapshot(&args.repo)?;
    store::require(&snapshot.root)?;
    let kind = Kind::parse(&args.kind)?;
    let id = Uuid::now_v7().to_string();
    let created_at = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let status = match kind {
        Kind::Decision => Some(DecisionStatus::Active),
        Kind::Learning => None,
        Kind::Journal => unreachable!("journal entries use the journal command"),
    };
    let superseded = store::active_decisions(&snapshot.root, &args.supersedes)?;
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
        branch: snapshot.branch,
    };
    let path = store::write(&snapshot.root, kind, &record.id, &record.render())?;
    store::mark_superseded(&superseded, &record.id)?;
    println!("Created {}: {}", kind.as_str(), display_path(&path));
    println!("Record ID: {}", record.id);
    println!("Add detailed context directly to the Markdown body when useful.");
    if !superseded.is_empty() {
        println!("Superseded {} earlier decision(s).", superseded.len());
    }
    Ok(())
}

fn journal(args: JournalArgs) -> Result<(), String> {
    let root = git::repository_root(&args.repo)?;
    store::require(&root)?;
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let created_at = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    let update = store::update_journal(
        &root,
        &date,
        &created_at,
        &args.done,
        &args.questions,
        &args.next,
    )?;
    println!("Updated journal: {}", display_path(&update.path));
    if update.added == 0 {
        println!("No new bullets; exact duplicates were already present.");
    } else {
        println!("Added {} bullet(s).", update.added);
    }
    Ok(())
}

fn resume(path: std::path::PathBuf) -> Result<(), String> {
    let root = git::repository_root(&path)?;
    let Some((journal_path, content)) = store::latest_journal(&root)? else {
        println!("No journal entries yet.");
        println!("Use `tincan journal --done <text>` as meaningful work develops.");
        return Ok(());
    };
    println!("Latest journal: {}\n", display_path(&journal_path));
    print!("{content}");
    Ok(())
}

fn search(path: std::path::PathBuf, query: &str) -> Result<(), String> {
    let root = git::repository_root(&path)?;
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
    let root = git::repository_root(&path)?;
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

fn check(path: std::path::PathBuf) -> Result<(), String> {
    let root = git::repository_root(&path)?;
    let changed = git::changed_files(&root)?;
    if changed.is_empty() {
        println!("No changed files.");
        return Ok(());
    }

    println!("Changed files:");
    for file in &changed {
        println!("  {file}");
    }

    let mut related = Vec::new();
    for document in store::scan(&root)? {
        let matched: Vec<_> = document
            .files
            .iter()
            .filter(|affected| {
                changed
                    .iter()
                    .any(|file| paths_overlap(file, affected.as_str()))
            })
            .cloned()
            .collect();
        if !matched.is_empty() {
            related.push((document, matched));
        }
    }

    if related.is_empty() {
        println!("\nNo path-related Tincan history found.");
        return Ok(());
    }

    println!("\nRelated Tincan history:");
    for (document, matched) in related {
        let label = document
            .status
            .as_deref()
            .map(|status| format!("{} / {status}", document.kind))
            .unwrap_or_else(|| document.kind.clone());
        println!(
            "  {} [{}]\n    matched: {}\n    {}",
            document.heading,
            label,
            matched.join(", "),
            display_path(&document.path)
        );
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
