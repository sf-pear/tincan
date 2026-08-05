use crate::model::{DecisionStatus, Kind};
use crate::util::slug;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const AGENT_GUIDE: &str = r#"# Tincan Agent Guide

Tincan is this checkout's private development memory. Markdown under `.tincan/`
is canonical and locally excluded from Git by default.

## Resume work

1. Run `tincan resume` to read the latest daily journal.
2. Run `tincan check --changed` when touching existing work.
3. Search focused terms with `tincan search "<file, feature, or concept>"`.
4. Use `tincan show <record-id>` only for relevant full records.

## Classify information

- Add a concise `journal --done` bullet after a meaningful implementation.
- Add unresolved matters with `journal --question` and concrete unfinished work
  with `journal --next`. Journal entries answer what happened today or what is
  currently open; they are not durable conclusions.
- Record a decision only when a choice is accepted and constrains future work.
- Use `--supersedes <decision-id>` when a new decision replaces an old one.
- Do not pass decision status; Tincan manages `active` and `superseded`.
- Record a learning only when evidence supports knowledge useful beyond today's
  work. A failed approach is a learning only when it yields a durable conclusion.
- Do not pass status for learnings; they do not have one.
- Do not turn open questions, todos, progress, or speculation into decisions or
  learnings. Keep those in the journal.
- Keep journal bullets brief. Put full reasoning in decisions and learnings
  rather than duplicating it in the journal.
- If record validation fails, correct the arguments described by the error and
  retry. Do not bypass validation by writing the record file manually.

Do not store credentials, customer data, or complete raw transcripts in Tincan.
Raw session text is evidence, not accepted project truth.
"#;

const DIRECTORIES: [&str; 3] = ["decisions", "learnings", "journal"];

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub id: String,
    pub title: String,
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
        fs::write(
            &config,
            "# Tincan repository configuration\nversion = 1\nstorage = \"markdown\"\n",
        )
        .map_err(|error| format!("cannot write {}: {error}", config.display()))?;
    }
    let guide = root.join("AGENT_GUIDE.md");
    if !guide.exists() {
        fs::write(&guide, AGENT_GUIDE)
            .map_err(|error| format!("cannot write {}: {error}", guide.display()))?;
    }
    Ok(root)
}

pub fn ensure_git_excluded(path: &Path) -> Result<bool, String> {
    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    if existing
        .lines()
        .any(|line| matches!(line.trim(), ".tincan" | ".tincan/" | "/.tincan/"))
    {
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
    updated.push_str(".tincan/\n");
    fs::write(path, updated)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(true)
}

pub fn require(repo: &Path) -> Result<PathBuf, String> {
    let root = repo.join(".tincan");
    if !root.join("config.toml").is_file() {
        return Err(format!(
            "{} is not initialized; run `tincan init {}`",
            repo.display(),
            repo.display()
        ));
    }
    for directory in DIRECTORIES {
        fs::create_dir_all(root.join(directory))
            .map_err(|error| format!("cannot create .tincan/{directory}: {error}"))?;
    }
    Ok(root)
}

pub fn write(
    repo: &Path,
    kind: Kind,
    timestamp: u64,
    title: &str,
    content: &str,
) -> Result<PathBuf, String> {
    let root = require(repo)?;
    let filename = format!("{timestamp}-{}.md", slug(title));
    let path = root.join(kind.directory()).join(filename);
    if path.exists() {
        return Err(format!("record already exists: {}", path.display()));
    }
    fs::write(&path, content)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(path)
}

pub struct JournalUpdate {
    pub path: PathBuf,
    pub added: usize,
}

pub fn update_journal(
    repo: &Path,
    date: &str,
    created_at: &str,
    done: &[String],
    questions: &[String],
    next: &[String],
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
    let mut question_items = markdown_bullets(&existing, "## Open questions");
    let mut next_items = markdown_bullets(&existing, "## Next");
    let mut added = 0;

    for item in clean_bullets(done) {
        if !done_items.contains(&item) {
            done_items.push(item.clone());
            added += 1;
        }
        next_items.retain(|next_item| next_item != &item);
    }
    for item in clean_bullets(questions) {
        if !question_items.contains(&item) {
            question_items.push(item);
            added += 1;
        }
    }
    for item in clean_bullets(next) {
        if !done_items.contains(&item) && !next_items.contains(&item) {
            next_items.push(item);
            added += 1;
        }
    }

    let content = render_journal(
        date,
        &original_created_at,
        &done_items,
        &question_items,
        &next_items,
    );
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
        if in_section && let Some(value) = line.strip_prefix("- ") {
            values.push(value.trim().to_string());
        }
    }
    values
}

fn render_journal(
    date: &str,
    created_at: &str,
    done: &[String],
    questions: &[String],
    next: &[String],
) -> String {
    let mut output = format!(
        "---\nid: {}\ntype: \"journal\"\ncreated_at: {}\ntitle: {}\n---\n\n# {date}\n",
        crate::util::yaml_string(&format!("journal-{date}")),
        crate::util::yaml_string(created_at),
        crate::util::yaml_string(date),
    );
    for (heading, values) in [
        ("Done", done),
        ("Open questions", questions),
        ("Next", next),
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
        let frontmatter = read_frontmatter(&file_path)?;
        output.push(parse_document(file_path, frontmatter)?);
    }
    Ok(())
}

fn parse_document(path: PathBuf, text: String) -> Result<Document, String> {
    let kind = scalar(&text, "type").unwrap_or_default();
    let status = scalar(&text, "status");
    validate_record_status(&path, &kind, status.as_deref())?;
    Ok(Document {
        id: scalar(&text, "id").unwrap_or_default(),
        title: scalar(&text, "title").unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }),
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
        _ => Ok(()),
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
            title: document.title.clone(),
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
    for decision in decisions {
        let text = read_document(decision)?;
        let updated = add_supersession(&text, replacement_id)?;
        fs::write(&decision.path, updated)
            .map_err(|error| format!("cannot write {}: {error}", decision.path.display()))?;
    }
    Ok(())
}

fn read_frontmatter(path: &Path) -> Result<String, String> {
    let file =
        fs::File::open(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let Some(first) = lines.next() else {
        return Ok(String::new());
    };
    let first = first.map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if first != "---" {
        return Ok(String::new());
    }

    let mut output = String::from("---\n");
    for line in lines {
        let line = line.map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        output.push_str(&line);
        output.push('\n');
        if line == "---" {
            break;
        }
    }
    Ok(output)
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

    #[test]
    fn parses_frontmatter_index_fields() {
        let document = parse_document(
            PathBuf::from("test.md"),
            "---\nid: \"a\"\ntype: \"decision\"\nstatus: \"active\"\ntitle: \"Paging\"\nfiles:\n  - \"src/a.rs\"\ntopics:\n  - \"performance\"\nrelated:\nsupersedes:\nsuperseded_by:\n---\n"
                .to_string(),
        )
        .unwrap();
        assert_eq!(document.id, "a");
        assert_eq!(document.files, vec!["src/a.rs"]);
        assert_eq!(document.topics, vec!["performance"]);
    }

    #[test]
    fn rejects_invented_decision_statuses_and_status_on_learnings() {
        let invented = parse_document(
            PathBuf::from("decision.md"),
            "---\nid: \"a\"\ntype: \"decision\"\nstatus: \"mostly-active\"\n---\n".to_string(),
        )
        .unwrap_err();
        assert!(invented.contains("invalid decision status"));

        let learning = parse_document(
            PathBuf::from("learning.md"),
            "---\nid: \"b\"\ntype: \"learning\"\nstatus: \"accepted\"\n---\n".to_string(),
        )
        .unwrap_err();
        assert!(learning.contains("learnings must not have a status"));

        parse_document(
            PathBuf::from("learning.md"),
            "---\nid: \"b\"\ntype: \"learning\"\ntitle: \"A reusable fact\"\n---\n".to_string(),
        )
        .unwrap();
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

        assert!(ensure_git_excluded(&path).unwrap());
        assert!(!ensure_git_excluded(&path).unwrap());

        let content = fs::read_to_string(path).unwrap();
        assert!(content.starts_with("target/\n"));
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
        assert!(root.join("AGENT_GUIDE.md").is_file());
        for directory in DIRECTORIES {
            assert!(root.join(directory).is_dir());
        }

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
            &[],
            &["Should this be shared?".into()],
            &["Implement journal".into()],
        )
        .unwrap();
        update_journal(
            &repo,
            "2026-08-05",
            "2026-08-05T10:00:00+02:00",
            &["Implement journal".into()],
            &["Should this be shared?".into()],
            &[],
        )
        .unwrap();

        let path = repo.join(".tincan/journal/2026-08-05.md");
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.matches("- Implement journal").count(), 1);
        assert_eq!(content.matches("- Should this be shared?").count(), 1);
        assert!(content.contains("created_at: \"2026-08-05T09:00:00+02:00\""));
        fs::remove_dir_all(repo).unwrap();
    }
}
