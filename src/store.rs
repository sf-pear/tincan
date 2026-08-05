use crate::model::{DecisionStatus, Kind};
use crate::util::slug;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const AGENT_GUIDE: &str = r#"# Tincan Agent Guide

Tincan is this repository's development ledger. Records live as Markdown under
`.tincan/` and preserve evidence that should survive changing agents and tools.

## Before changing code

1. Run `tincan check --changed`.
2. Run `tincan search "<relevant feature, file, or concept>"`.
3. Use `tincan show <record-id>` to read only relevant full records.

## During work

- Record accepted choices with `tincan record decision`.
- Use `--supersedes <decision-id>` when a new decision replaces an old one.
- Do not pass decision status; Tincan manages `active` and `superseded`.
- Record reusable, evidence-supported findings with `tincan record learning`.
- Do not pass status for learnings; they do not have one.
- Write the reasoning and outcome together in `--note`.
- Use repository-relative paths in repeatable `--file` options.
- If record validation fails, correct the arguments described by the error and
  retry. Do not bypass validation by writing the record file manually.

## Before finishing

Run the relevant verification commands, then record only durable decisions and
reusable, evidence-supported learnings created by the work.

Do not store credentials, customer data, or complete raw transcripts in Tincan.
Raw session text is evidence, not accepted project truth.
"#;

const AGENTS_INSTRUCTION: &str =
    "Before starting or finishing development work, read and follow `.tincan/AGENT_GUIDE.md`.";

const DIRECTORIES: [&str; 2] = ["decisions", "learnings"];

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

pub fn install_agent_guidance(repo: &Path) -> Result<(PathBuf, bool), String> {
    let path = repo.join("AGENTS.md");
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    if existing
        .lines()
        .any(|line| line.trim() == AGENTS_INSTRUCTION)
    {
        return Ok((path, false));
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str(AGENTS_INSTRUCTION);
    updated.push('\n');
    fs::write(&path, updated)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok((path, true))
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
    fn installs_agent_guidance_once_without_replacing_existing_content() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-agent-guidance-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        let path = repo.join("AGENTS.md");
        fs::write(&path, "# Existing instructions\n").unwrap();

        install_agent_guidance(&repo).unwrap();
        install_agent_guidance(&repo).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.starts_with("# Existing instructions\n"));
        assert_eq!(content.matches(AGENTS_INSTRUCTION).count(), 1);
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
}
