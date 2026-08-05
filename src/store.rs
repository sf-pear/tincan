use crate::model::Kind;
use crate::util::slug;
use std::fs;
use std::path::{Path, PathBuf};

const AGENT_GUIDE: &str = r#"# Tincan Agent Guide

Tincan is this repository's development ledger. Records live as Markdown under
`.tincan/` and preserve evidence that should survive changing agents and tools.

## Before changing code

1. Run `tincan check --changed`.
2. Run `tincan search "<relevant feature, file, or concept>"`.
3. Read related records before proceeding.

## During work

- Record meaningful experiments with `tincan record attempt`.
- Preserve failed and inconclusive attempts, not only successes.
- Record accepted choices with `tincan record decision`.
- Record reusable, evidence-supported findings with `tincan record learning`.
- Use repository-relative paths in `--affects`.

## Before finishing

1. Run the relevant verification commands.
2. Use `tincan capture --title "<work summary>"` for substantial sessions.
3. Use `tincan handoff` when work remains or another agent will continue.

Do not store credentials, customer data, or complete raw transcripts in Tincan.
Draft session text is evidence, not accepted project truth.
"#;

const AGENTS_INSTRUCTION: &str =
    "Before starting or finishing development work, read and follow `.tincan/AGENT_GUIDE.md`.";

const DIRECTORIES: [&str; 6] = [
    "attempts",
    "decisions",
    "learnings",
    "handoffs",
    "sessions",
    "field-notes",
];

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub affects: Vec<String>,
    pub topics: Vec<String>,
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
        let text = fs::read_to_string(&file_path)
            .map_err(|error| format!("cannot read {}: {error}", file_path.display()))?;
        output.push(parse_document(file_path, text));
    }
    Ok(())
}

fn parse_document(path: PathBuf, text: String) -> Document {
    Document {
        id: scalar(&text, "id").unwrap_or_default(),
        title: scalar(&text, "title").unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }),
        kind: scalar(&text, "type").unwrap_or_default(),
        status: scalar(&text, "status").unwrap_or_default(),
        affects: yaml_list(&text, "affects"),
        topics: yaml_list(&text, "topics"),
        path,
        text,
    }
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
            "---\nid: \"a\"\ntype: \"attempt\"\nstatus: \"failed\"\ntitle: \"Paging\"\naffects:\n  - \"src/a.rs\"\ntopics:\n  - \"performance\"\n---\n"
                .to_string(),
        );
        assert_eq!(document.id, "a");
        assert_eq!(document.affects, vec!["src/a.rs"]);
        assert_eq!(document.topics, vec!["performance"]);
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
}
