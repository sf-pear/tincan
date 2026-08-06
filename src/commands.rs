use crate::cli::{self, Command, JournalArgs, RecordArgs};
use crate::git;
use crate::model::{DecisionStatus, Kind, Record};
use crate::skill::{self, InstallOutcome};
use crate::store;
use crate::util::{display_path, slug, unix_timestamp};
use chrono::{Local, SecondsFormat};
use std::collections::BTreeMap;

pub fn run(command: Result<Command, String>) -> Result<(), String> {
    match command? {
        Command::Help => {
            print!("{}", cli::help());
            Ok(())
        }
        Command::Version => {
            println!("tincan {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Init { repo } => init(repo),
        Command::Inspect { repo } => inspect(repo),
        Command::Record(args) => record(args),
        Command::Journal(args) => journal(args),
        Command::Resume { repo } => resume(repo),
        Command::Search { repo, query } => search(repo, &query),
        Command::Show { repo, id } => show(repo, &id),
        Command::CheckChanged { repo } => check_changed(repo),
        Command::SkillInstall { path, force } => install_skill(path, force),
    }
}

fn install_skill(path: Option<std::path::PathBuf>, force: bool) -> Result<(), String> {
    match skill::install(path.as_deref(), force)? {
        InstallOutcome::Installed(path) => {
            println!("Installed Tincan skill at {}", display_path(&path));
            println!("Restart or reload the agent harness to discover it.");
        }
        InstallOutcome::AlreadyCurrent(path) => {
            println!("Tincan skill is already current at {}", display_path(&path));
        }
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
    println!("Initialized Tincan at {}", display_path(&tincan));
    if excluded {
        println!("Kept .tincan private through Git's local exclude file.");
    } else {
        println!(".tincan is already excluded from Git locally.");
    }
    Ok(())
}

fn inspect(path: std::path::PathBuf) -> Result<(), String> {
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
    let timestamp = unix_timestamp()?;
    let kind = Kind::parse(&args.kind)?;
    let id = format!("{}-{timestamp}-{}", kind.as_str(), slug(&args.title));
    let status = match kind {
        Kind::Decision => Some(DecisionStatus::Active),
        Kind::Learning => None,
        Kind::Journal => unreachable!("journal entries use the journal command"),
    };
    let superseded = store::active_decisions(&snapshot.root, &args.supersedes)?;
    let record = Record {
        id,
        kind,
        timestamp,
        title: args.title.clone(),
        status,
        note: args.note,
        files: args.files,
        topics: args.topics,
        evidence: args.evidence,
        related: args.related,
        supersedes: args.supersedes,
        branch: snapshot.branch,
    };
    let path = store::write(
        &snapshot.root,
        kind,
        timestamp,
        &args.title,
        &record.render(),
    )?;
    store::mark_superseded(&superseded, &record.id)?;
    println!("Recorded {}: {}", kind.as_str(), display_path(&path));
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
    let matches: Vec<_> = store::scan(&root)?
        .into_iter()
        .filter(|document| metadata_text(document).to_lowercase().contains(&query))
        .collect();
    if matches.is_empty() {
        println!("No Tincan records matched.");
        return Ok(());
    }
    for document in matches {
        print_document_summary(&document);
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

fn metadata_text(document: &store::Document) -> String {
    [
        vec![
            document.id.clone(),
            document.title.clone(),
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

fn print_document_summary(document: &store::Document) {
    let label = document
        .status
        .as_deref()
        .map(|status| format!("{} / {status}", document.kind))
        .unwrap_or_else(|| document.kind.clone());
    println!("{} [{label}]", document.title);
    println!("  id: {}", document.id);
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

fn check_changed(path: std::path::PathBuf) -> Result<(), String> {
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
            document.title,
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

    #[test]
    fn matches_files_and_directories() {
        assert!(paths_overlap("src/feature/a.rs", "src/feature"));
        assert!(paths_overlap("src/feature", "src/feature/a.rs"));
        assert!(!paths_overlap("src/a.rs", "src/b.rs"));
    }
}
