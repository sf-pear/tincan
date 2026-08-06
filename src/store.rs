use crate::model::{DecisionStatus, Kind};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const AGENT_GUIDE: &str = r#"# Tincan Agent Guide

Tincan is this workspace's private development memory. The workspace may contain
zero, one, or several Git repositories. Markdown under `.tincan/` is canonical.

## Start work

1. Run `tincan plan` to read the living outcome-level direction.
2. Run `tincan resume` to read the latest daily journal.
3. Run `tincan changes` when file-linked history may matter.
4. Use focused `search` queries and `show` only for relevant full records.

## Maintain memory

- Edit `.tincan/plan.md` directly. Keep current outcomes and ideas, not an
  implementation checklist. Remove completed items; the journal keeps history.
- Use short journal bullets for done, decisions, learnings, planned work, open
  questions, and the next starting point.
- Run `tincan decide <statement>` only for an accepted choice that constrains
  future work. Use `--supersedes <uuid>` when replacing an active decision.
- Run `tincan learn <statement>` only for an evidence-supported conclusion that
  remains useful beyond the current session.
- Use workspace-relative `--file` paths. Let Tincan create UUIDs and frontmatter,
  then add useful detail below the H1.

## Wrap up

When asked to wrap up, finish for today, record learnings, or prepare for
tomorrow, reconcile the conversation, plan, journal, decisions, and learnings.
Update the plan, write concise journal bullets, then run `tincan resume` so the
user can review the finalized journal. Distinguish implemented work from work
that was only decided or planned.

Do not store credentials, customer data, or raw transcripts.
"#;

const DIRECTORIES: [&str; 3] = ["decisions", "learnings", "journal"];
const CONFIG: &str = "# Tincan workspace configuration\nversion = 2\nstorage = \"markdown\"\n";
const PLAN: &str = "# Plan\n\n## Planned\n\n<!-- none -->\n\n## Ideas\n\n<!-- none -->\n";

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub id: String,
    pub heading: String,
    pub body: String,
    pub kind: String,
    pub status: Option<String>,
    pub files: Vec<String>,
    pub topics: Vec<String>,
    pub related: Vec<String>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
}

pub fn initialize(repo: &Path) -> Result<PathBuf, String> {
    let root = repo.join(".tincan");
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    for directory in DIRECTORIES {
        fs::create_dir_all(root.join(directory))
            .map_err(|error| format!("cannot create .tincan/{directory}: {error}"))?;
    }
    let config = root.join("config.toml");
    if !config.exists() {
        fs::write(&config, CONFIG)
            .map_err(|error| format!("cannot write {}: {error}", config.display()))?;
    } else {
        validate_config(&config)?;
    }
    let guide = root.join("AGENT_GUIDE.md");
    if !guide.exists() {
        fs::write(&guide, AGENT_GUIDE)
            .map_err(|error| format!("cannot write {}: {error}", guide.display()))?;
    }
    let plan = root.join("plan.md");
    if !plan.exists() {
        fs::write(&plan, PLAN)
            .map_err(|error| format!("cannot write {}: {error}", plan.display()))?;
    }
    Ok(root)
}

pub fn ensure_git_excluded(path: &Path, pattern: &str) -> Result<bool, String> {
    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    if existing.lines().any(|line| line.trim() == pattern) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(pattern);
    updated.push('\n');
    fs::write(path, updated)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(true)
}

pub fn require(repo: &Path) -> Result<PathBuf, String> {
    let root = repo.join(".tincan");
    let config = root.join("config.toml");
    if !config.is_file() {
        return Err(format!(
            "{} is not initialized; run `tincan init {}`",
            repo.display(),
            repo.display()
        ));
    }
    validate_config(&config)?;
    for directory in DIRECTORIES {
        fs::create_dir_all(root.join(directory))
            .map_err(|error| format!("cannot create .tincan/{directory}: {error}"))?;
    }
    let plan = root.join("plan.md");
    if !plan.exists() {
        fs::write(&plan, PLAN)
            .map_err(|error| format!("cannot write {}: {error}", plan.display()))?;
    }
    Ok(root)
}

pub fn read_plan(repo: &Path) -> Result<(PathBuf, String), String> {
    let path = require(repo)?.join("plan.md");
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok((path, content))
}

fn validate_config(path: &Path) -> Result<(), String> {
    let config = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if config.lines().any(|line| line.trim() == "version = 2") {
        Ok(())
    } else {
        Err(format!(
            "{} uses an unsupported Tincan storage version; expected version 2",
            path.parent().unwrap_or(path).display()
        ))
    }
}

pub fn write(repo: &Path, kind: Kind, id: &str, content: &str) -> Result<PathBuf, String> {
    let root = require(repo)?;
    let filename = format!("{id}.md");
    let path = root.join(kind.directory()).join(filename);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                format!("record already exists: {}", path.display())
            } else {
                format!("cannot write {}: {error}", path.display())
            }
        })?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(path)
}

pub struct JournalUpdate {
    pub path: PathBuf,
    pub added: usize,
}

pub struct JournalSections<'a> {
    pub done: &'a [String],
    pub decisions: &'a [String],
    pub learnings: &'a [String],
    pub planned: &'a [String],
    pub questions: &'a [String],
    pub next: &'a [String],
}

pub fn update_journal(
    repo: &Path,
    date: &str,
    created_at: &str,
    sections: JournalSections<'_>,
) -> Result<JournalUpdate, String> {
    let root = require(repo)?;
    let path = root.join("journal").join(format!("{date}.md"));
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let original_created_at = scalar(&existing, "created_at").unwrap_or_else(|| created_at.into());
    let mut done_items = markdown_bullets(&existing, "## Done");
    let mut decision_items = markdown_bullets(&existing, "## Decisions");
    let mut learning_items = markdown_bullets(&existing, "## Learnings");
    let mut planned_items = markdown_bullets(&existing, "## Planned");
    let mut question_items = markdown_bullets(&existing, "## Open questions");
    let mut next_items = markdown_bullets(&existing, "## Next");
    let mut added = 0;

    for item in clean_bullets(sections.done) {
        if !done_items.contains(&item) {
            done_items.push(item.clone());
            added += 1;
        }
        next_items.retain(|next_item| next_item != &item);
    }
    for (values, items) in [
        (sections.decisions, &mut decision_items),
        (sections.learnings, &mut learning_items),
        (sections.planned, &mut planned_items),
    ] {
        for item in clean_bullets(values) {
            if !items.contains(&item) {
                items.push(item);
                added += 1;
            }
        }
    }
    for item in clean_bullets(sections.questions) {
        if !question_items.contains(&item) {
            question_items.push(item);
            added += 1;
        }
    }
    for item in clean_bullets(sections.next) {
        if !done_items.contains(&item) && !next_items.contains(&item) {
            next_items.push(item);
            added += 1;
        }
    }

    let rendered = JournalSections {
        done: &done_items,
        decisions: &decision_items,
        learnings: &learning_items,
        planned: &planned_items,
        questions: &question_items,
        next: &next_items,
    };
    let content = render_journal(date, &original_created_at, rendered);
    fs::write(&path, content)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(JournalUpdate { path, added })
}

pub fn latest_journal(repo: &Path) -> Result<Option<(PathBuf, String)>, String> {
    let root = require(repo)?;
    let directory = root.join("journal");
    let mut paths = fs::read_dir(&directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    paths.sort();
    let Some(path) = paths.pop() else {
        return Ok(None);
    };
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(Some((path, content)))
}

fn clean_bullets(values: &[String]) -> impl Iterator<Item = String> + '_ {
    values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn markdown_bullets(text: &str, heading: &str) -> Vec<String> {
    let mut in_section = false;
    let mut values = Vec::new();
    for line in text.lines() {
        if line == heading {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            if let Some(value) = line.strip_prefix("- ") {
                values.push(value.trim().to_string());
            }
        }
    }
    values
}

fn render_journal(date: &str, created_at: &str, sections: JournalSections<'_>) -> String {
    let mut output = format!(
        "---\nid: {}\ntype: \"journal\"\ncreated_at: {}\n---\n\n# {date}\n",
        crate::util::yaml_string(&format!("journal-{date}")),
        crate::util::yaml_string(created_at),
    );
    for (heading, values) in [
        ("Done", sections.done),
        ("Decisions", sections.decisions),
        ("Learnings", sections.learnings),
        ("Planned", sections.planned),
        ("Open questions", sections.questions),
        ("Next", sections.next),
    ] {
        output.push_str(&format!("\n## {heading}\n\n"));
        if values.is_empty() {
            output.push_str("<!-- none -->\n");
        } else {
            for value in values {
                output.push_str("- ");
                output.push_str(value);
                output.push('\n');
            }
        }
    }
    output
}

pub fn scan(repo: &Path) -> Result<Vec<Document>, String> {
    let root = require(repo)?;
    let mut documents = Vec::new();
    for directory in DIRECTORIES {
        collect_documents(&root.join(directory), &mut documents)?;
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

fn collect_documents(path: &Path, output: &mut Vec<Document>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_path = entry.path();
        if file_path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(&file_path)
            .map_err(|error| format!("cannot read {}: {error}", file_path.display()))?;
        output.push(parse_document(file_path, text)?);
    }
    Ok(())
}

fn parse_document(path: PathBuf, text: String) -> Result<Document, String> {
    let kind = scalar(&text, "type").unwrap_or_default();
    let status = scalar(&text, "status");
    validate_record_status(&path, &kind, status.as_deref())?;
    let id = scalar(&text, "id").unwrap_or_default();
    if id.is_empty() {
        return Err(format!(
            "invalid frontmatter in {}: id is required",
            path.display()
        ));
    }
    if matches!(kind.as_str(), "decision" | "learning") {
        uuid::Uuid::parse_str(&id).map_err(|_| {
            format!(
                "invalid frontmatter in {}: decision and learning ids must be UUIDs",
                path.display()
            )
        })?;
        if scalar(&text, "created_at").is_none() {
            return Err(format!(
                "invalid frontmatter in {}: created_at is required",
                path.display()
            ));
        }
        for field in ["related", "supersedes", "superseded_by"] {
            for related_id in yaml_list(&text, field) {
                if uuid::Uuid::parse_str(&related_id).is_err() {
                    return Err(format!(
                        "invalid frontmatter in {}: {field} values must be UUIDs",
                        path.display()
                    ));
                }
            }
        }
    }
    let body = markdown_body(&text);
    let parsed_heading = body
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|heading| !heading.is_empty())
        .map(str::to_string);
    if matches!(kind.as_str(), "decision" | "learning") && parsed_heading.is_none() {
        return Err(format!(
            "invalid Markdown in {}: decision and learning records require an H1 heading",
            path.display()
        ));
    }
    let heading = parsed_heading.unwrap_or_else(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });
    if matches!(kind.as_str(), "decision" | "learning")
        && path.file_stem().and_then(|value| value.to_str()) != Some(id.as_str())
    {
        return Err(format!(
            "invalid frontmatter in {}: UUID filename and id do not match",
            path.display()
        ));
    }
    Ok(Document {
        id,
        heading,
        body,
        kind,
        status,
        files: yaml_list(&text, "files"),
        topics: yaml_list(&text, "topics"),
        related: yaml_list(&text, "related"),
        supersedes: yaml_list(&text, "supersedes"),
        superseded_by: yaml_list(&text, "superseded_by"),
        path,
    })
}

fn validate_record_status(path: &Path, kind: &str, status: Option<&str>) -> Result<(), String> {
    let result = match (kind, status) {
        ("decision", Some(value)) => DecisionStatus::parse(value).map(|_| ()),
        ("decision", None) => Err("decision status is required".to_string()),
        ("learning", None) => Ok(()),
        ("learning", Some(_)) => Err("learnings must not have a status".to_string()),
        ("journal", None) => Ok(()),
        ("journal", Some(_)) => Err("journal entries must not have a status".to_string()),
        _ => Err(format!("unsupported record type: {kind}")),
    };
    result.map_err(|error| format!("invalid frontmatter in {}: {error}", path.display()))
}

pub fn read_document(document: &Document) -> Result<String, String> {
    fs::read_to_string(&document.path)
        .map_err(|error| format!("cannot read {}: {error}", document.path.display()))
}

pub fn active_decisions(repo: &Path, ids: &[String]) -> Result<Vec<Document>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let documents = scan(repo)?;
    let mut decisions = Vec::new();
    for id in ids {
        let document = documents
            .iter()
            .find(|document| document.id == *id)
            .ok_or_else(|| format!("no Tincan record found with id {id}"))?;
        if document.kind != "decision" {
            return Err(format!("{id} is not a decision"));
        }
        if document.status.as_deref() != Some("active") {
            return Err(format!(
                "decision {id} is {}; only active decisions can be superseded",
                document.status.as_deref().unwrap_or("no status")
            ));
        }
        decisions.push(Document {
            path: document.path.clone(),
            id: document.id.clone(),
            heading: document.heading.clone(),
            body: document.body.clone(),
            kind: document.kind.clone(),
            status: document.status.clone(),
            files: document.files.clone(),
            topics: document.topics.clone(),
            related: document.related.clone(),
            supersedes: document.supersedes.clone(),
            superseded_by: document.superseded_by.clone(),
        });
    }
    Ok(decisions)
}

pub fn mark_superseded(decisions: &[Document], replacement_id: &str) -> Result<(), String> {
    let updates = decisions
        .iter()
        .map(|decision| {
            let original = read_document(decision)?;
            let updated = add_supersession(&original, replacement_id)?;
            Ok((decision.path.clone(), original, updated))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut written: Vec<(PathBuf, String)> = Vec::new();
    for (path, original, updated) in &updates {
        if let Err(error) = fs::write(path, updated) {
            let mut rollback_errors = Vec::new();
            for (written_path, written_original) in written.into_iter().rev() {
                if let Err(rollback_error) = fs::write(&written_path, written_original) {
                    rollback_errors.push(format!("{}: {rollback_error}", written_path.display()));
                }
            }
            let mut message = format!("cannot write {}: {error}", path.display());
            if !rollback_errors.is_empty() {
                message.push_str(&format!(
                    "; rollback also failed for {}",
                    rollback_errors.join(", ")
                ));
            }
            return Err(message);
        }
        written.push((path.clone(), original.clone()));
    }
    Ok(())
}

fn markdown_body(text: &str) -> String {
    let mut lines = text.lines();
    if lines.next() != Some("---") {
        return text.to_string();
    }
    for line in &mut lines {
        if line == "---" {
            return lines.collect::<Vec<_>>().join("\n");
        }
    }
    String::new()
}

fn add_supersession(text: &str, replacement_id: &str) -> Result<String, String> {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    if lines.first().map(String::as_str) != Some("---") {
        return Err("decision has no YAML frontmatter".to_string());
    }
    let end = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| line.as_str() == "---")
        .map(|(index, _)| index)
        .ok_or_else(|| "decision has unterminated YAML frontmatter".to_string())?;

    let status = lines[..end]
        .iter()
        .position(|line| line.starts_with("status:"))
        .ok_or_else(|| "decision has no status field".to_string())?;
    lines[status] = "status: \"superseded\"".to_string();

    let marker = lines[..end]
        .iter()
        .position(|line| line == "superseded_by:");
    let insertion = if let Some(marker) = marker {
        let mut index = marker + 1;
        while index < end && lines[index].starts_with("  - ") {
            if unquote(lines[index].trim_start_matches("  - ").trim()) == replacement_id {
                return Ok(format!("{}\n", lines.join("\n")));
            }
            index += 1;
        }
        index
    } else {
        lines.insert(end, "superseded_by:".to_string());
        end + 1
    };
    lines.insert(
        insertion,
        format!("  - {}", crate::util::yaml_string(replacement_id)),
    );
    Ok(format!("{}\n", lines.join("\n")))
}

fn scalar(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    frontmatter_lines(text).find_map(|line| {
        line.strip_prefix(&prefix)
            .map(|value| unquote(value.trim()))
    })
}

fn yaml_list(text: &str, key: &str) -> Vec<String> {
    let marker = format!("{key}:");
    let mut found = false;
    let mut values = Vec::new();
    for line in frontmatter_lines(text) {
        if line == marker {
            found = true;
            continue;
        }
        if found {
            if let Some(value) = line.strip_prefix("  - ") {
                values.push(unquote(value.trim()));
            } else {
                break;
            }
        }
    }
    values
}

fn frontmatter_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut lines = text.lines();
    let has_opening_marker = lines.next() == Some("---");
    lines
        .take_while(|line| *line != "---")
        .filter(move |_| has_opening_marker)
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_ID: &str = "019c4ea8-7e42-7b31-a211-8df9357d747c";

    #[test]
    fn parses_frontmatter_index_fields() {
        let document = parse_document(
            PathBuf::from(format!("{TEST_ID}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"decision\"\nstatus: \"active\"\ncreated_at: \"2026-08-06T10:00:00Z\"\nfiles:\n  - \"src/a.rs\"\ntopics:\n  - \"performance\"\nrelated:\nsupersedes:\nsuperseded_by:\n---\n\n# Paging did not help\n\nDetailed renderer evidence.\n"
            ),
        )
        .unwrap();
        assert_eq!(document.id, TEST_ID);
        assert_eq!(document.heading, "Paging did not help");
        assert!(document.body.contains("Detailed renderer evidence."));
        assert_eq!(document.files, vec!["src/a.rs"]);
        assert_eq!(document.topics, vec!["performance"]);
    }

    #[test]
    fn rejects_invented_decision_statuses_and_status_on_learnings() {
        let invented = parse_document(
            PathBuf::from(format!("{TEST_ID}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"decision\"\nstatus: \"mostly-active\"\ncreated_at: \"2026-08-06T10:00:00Z\"\n---\n\n# Choice\n"
            ),
        )
        .unwrap_err();
        assert!(invented.contains("invalid decision status"));

        let learning = parse_document(
            PathBuf::from(format!("{TEST_ID}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"learning\"\nstatus: \"accepted\"\ncreated_at: \"2026-08-06T10:00:00Z\"\n---\n\n# Fact\n"
            ),
        )
        .unwrap_err();
        assert!(learning.contains("learnings must not have a status"));

        parse_document(
            PathBuf::from(format!("{TEST_ID}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"learning\"\ncreated_at: \"2026-08-06T10:00:00Z\"\n---\n\n# A reusable fact\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn rejects_mismatched_uuid_filename_and_missing_heading() {
        let other = "019c4ea8-7e42-7b31-a211-8df9357d747d";
        let mismatch = parse_document(
            PathBuf::from(format!("{other}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"learning\"\ncreated_at: \"2026-08-06T10:00:00Z\"\n---\n\n# Fact\n"
            ),
        )
        .unwrap_err();
        assert!(mismatch.contains("UUID filename and id do not match"));

        let missing_heading = parse_document(
            PathBuf::from(format!("{TEST_ID}.md")),
            format!(
                "---\nid: \"{TEST_ID}\"\ntype: \"learning\"\ncreated_at: \"2026-08-06T10:00:00Z\"\n---\n\nBody without a heading.\n"
            ),
        )
        .unwrap_err();
        assert!(missing_heading.contains("require an H1 heading"));
    }

    #[test]
    fn excludes_tincan_locally_without_replacing_existing_content() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-git-exclude-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        let path = repo.join("exclude");
        fs::write(&path, "target/\n").unwrap();

        assert!(ensure_git_excluded(&path, "/.tincan/").unwrap());
        assert!(!ensure_git_excluded(&path, "/.tincan/").unwrap());

        let content = fs::read_to_string(path).unwrap();
        assert!(content.starts_with("target/\n"));
        assert!(content.contains("/.tincan/\n"));
        assert_eq!(content.matches(".tincan/").count(), 1);
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn initializes_tincan_layout() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-layout-{unique}"));
        fs::create_dir_all(&repo).unwrap();

        let root = initialize(&repo).unwrap();
        assert_eq!(root, repo.join(".tincan"));
        assert!(root.join("config.toml").is_file());
        assert_eq!(
            fs::read_to_string(root.join("config.toml")).unwrap(),
            CONFIG
        );
        assert!(root.join("AGENT_GUIDE.md").is_file());
        assert_eq!(fs::read_to_string(root.join("plan.md")).unwrap(), PLAN);
        for directory in DIRECTORIES {
            assert!(root.join(directory).is_dir());
        }

        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn rejects_older_storage_versions_instead_of_rewriting_them() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-old-layout-{unique}"));
        fs::create_dir_all(repo.join(".tincan")).unwrap();
        fs::write(
            repo.join(".tincan/config.toml"),
            "version = 1\nstorage = \"markdown\"\n",
        )
        .unwrap();

        let error = initialize(&repo).unwrap_err();
        assert!(error.contains("unsupported Tincan storage version"));
        assert!(
            fs::read_to_string(repo.join(".tincan/config.toml"))
                .unwrap()
                .contains("version = 1")
        );
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn marks_decision_as_superseded_without_rewriting_its_body() {
        let text = "---\nid: \"old\"\ntype: \"decision\"\nstatus: \"active\"\nsuperseded_by:\n---\n\n# Old choice\n\nOriginal reasoning.\n";
        let updated = add_supersession(text, "new").unwrap();

        assert!(updated.contains("status: \"superseded\""));
        assert!(updated.contains("superseded_by:\n  - \"new\""));
        assert!(updated.contains("# Old choice\n\nOriginal reasoning."));
    }

    #[test]
    fn updates_one_daily_journal_and_moves_completed_next_item() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-journal-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        initialize(&repo).unwrap();

        update_journal(
            &repo,
            "2026-08-05",
            "2026-08-05T09:00:00+02:00",
            JournalSections {
                done: &[],
                decisions: &[],
                learnings: &[],
                planned: &[],
                questions: &["Should this be shared?".into()],
                next: &["Implement journal".into()],
            },
        )
        .unwrap();
        update_journal(
            &repo,
            "2026-08-05",
            "2026-08-05T10:00:00+02:00",
            JournalSections {
                done: &["Implement journal".into()],
                decisions: &["Keep Markdown canonical".into()],
                learnings: &["Nested repositories need workspace-relative paths".into()],
                planned: &["Add a plan".into()],
                questions: &["Should this be shared?".into()],
                next: &[],
            },
        )
        .unwrap();

        let path = repo.join(".tincan/journal/2026-08-05.md");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.matches("- Implement journal").count(), 1);
        assert_eq!(content.matches("- Should this be shared?").count(), 1);
        assert!(content.contains("created_at: \"2026-08-05T09:00:00+02:00\""));
        assert!(content.contains("## Decisions\n\n- Keep Markdown canonical"));
        assert!(
            content.contains("## Learnings\n\n- Nested repositories need workspace-relative paths")
        );
        assert!(content.contains("## Planned\n\n- Add a plan"));
        fs::remove_dir_all(repo).unwrap();
    }
}
