use crate::cli::{self, CaptureArgs, Command, HandoffArgs, RecordArgs};
use crate::git;
use crate::model::{self, Kind, Record};
use crate::skill::{self, InstallOutcome};
use crate::store;
use crate::util::{display_path, slug, unix_timestamp};
use std::collections::BTreeMap;

pub fn run(command: Result<Command, String>) -> Result<(), String> {
    match command? {
        Command::Help => {
            print!("{}", cli::help());
            Ok(())
        }
        Command::Init { repo } => init(repo),
        Command::Inspect { repo } => inspect(repo),
        Command::Record(args) => record(args),
        Command::Capture(args) => capture(args),
        Command::Handoff(args) => handoff(args),
        Command::FieldNote {
            repo,
            source_id,
            title,
        } => field_note(repo, &source_id, title),
        Command::Search { repo, query } => search(repo, &query),
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
    let tincan = store::initialize(&root)?;
    println!("Initialized Tincan at {}", display_path(&tincan));
    let (agents, added) = store::install_agent_guidance(&root)?;
    let action = if added {
        "Added Tincan guidance to"
    } else {
        "Tincan guidance already present in"
    };
    println!("{action} {}", display_path(&agents));
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
    let snapshot = git::snapshot(&args.repo)?;
    store::require(&snapshot.root)?;
    let timestamp = unix_timestamp()?;
    let kind = Kind::parse(&args.kind)?;
    let id = format!("{}-{timestamp}-{}", kind.as_str(), slug(&args.title));
    let default_status = match kind {
        Kind::Attempt => "proposed",
        Kind::Decision | Kind::Learning => "accepted",
        _ => "draft",
    };
    let record = Record {
        id,
        kind,
        timestamp,
        title: args.title.clone(),
        status: args.status.unwrap_or_else(|| default_status.to_string()),
        summary: args.summary,
        result: args.result,
        affects: args.affects,
        topics: args.topics,
        evidence: args.evidence,
        related: args.related,
        branch: snapshot.branch,
    };
    let path = store::write(
        &snapshot.root,
        kind,
        timestamp,
        &args.title,
        &record.render(),
    )?;
    println!("Recorded {}: {}", kind.as_str(), display_path(&path));
    Ok(())
}

fn capture(args: CaptureArgs) -> Result<(), String> {
    let snapshot = git::snapshot(&args.repo)?;
    store::require(&snapshot.root)?;
    let timestamp = unix_timestamp()?;
    let title = args.title.unwrap_or_else(|| {
        if snapshot.branch.is_empty() {
            "Development session".to_string()
        } else {
            format!("{} development session", snapshot.branch)
        }
    });
    let id = format!("session-{timestamp}-{}", slug(&title));
    let content = model::render_session(&id, timestamp, &title, &snapshot);
    let path = store::write(&snapshot.root, Kind::Session, timestamp, &title, &content)?;
    println!("Captured session draft: {}", display_path(&path));
    println!("Review its outcomes, decisions, verification, and learning candidates.");
    Ok(())
}

fn handoff(args: HandoffArgs) -> Result<(), String> {
    let snapshot = git::snapshot(&args.repo)?;
    store::require(&snapshot.root)?;
    let timestamp = unix_timestamp()?;
    let title = args
        .title
        .unwrap_or_else(|| "Development handoff".to_string());
    let id = format!("handoff-{timestamp}-{}", slug(&title));
    let content = model::render_handoff(&id, timestamp, &title, &snapshot, &args.next);
    let path = store::write(&snapshot.root, Kind::Handoff, timestamp, &title, &content)?;
    println!("Created handoff: {}", display_path(&path));
    Ok(())
}

fn field_note(
    path: std::path::PathBuf,
    source_id: &str,
    title: Option<String>,
) -> Result<(), String> {
    let root = git::repository_root(&path)?;
    let source = store::scan(&root)?
        .into_iter()
        .find(|document| document.id == source_id)
        .ok_or_else(|| format!("no Tincan record found with id {source_id}"))?;
    let timestamp = unix_timestamp()?;
    let title = title.unwrap_or_else(|| source.title.clone());
    let id = format!("field-note-{timestamp}-{}", slug(&title));
    let content = model::render_field_note(&id, timestamp, &title, source_id, &source.title);
    let output = store::write(&root, Kind::FieldNote, timestamp, &title, &content)?;
    println!("Created field-note draft: {}", display_path(&output));
    println!("Rewrite it as a narrative and review private details before publishing.");
    Ok(())
}

fn search(path: std::path::PathBuf, query: &str) -> Result<(), String> {
    let root = git::repository_root(&path)?;
    let query = query.to_lowercase();
    let matches: Vec<_> = store::scan(&root)?
        .into_iter()
        .filter(|document| document.text.to_lowercase().contains(&query))
        .collect();
    if matches.is_empty() {
        println!("No Tincan records matched.");
        return Ok(());
    }
    for document in matches {
        let topics = if document.topics.is_empty() {
            String::new()
        } else {
            format!("\n  topics: {}", document.topics.join(", "))
        };
        println!(
            "{} [{} / {}]\n  id: {}{}\n  {}",
            document.title,
            document.kind,
            document.status,
            document.id,
            topics,
            display_path(&document.path)
        );
    }
    Ok(())
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
            .affects
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
        println!(
            "  {} [{} / {}]\n    matched: {}\n    {}",
            document.title,
            document.kind,
            document.status,
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
