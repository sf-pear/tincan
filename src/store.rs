use crate::model::{DecisionStatus, Kind};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DIRECTORIES: [&str; 3] = ["decisions", "learnings", "journal"];
const CONFIG: &str = "# Tincan workspace configuration\nversion = 2\nstorage = \"markdown\"\n";
const PLAN: &str = "# Plan\n\n## Planned\n\n<!-- none -->\n";
const LATER: &str = "# Later\n\n<!-- none -->\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Project,
    Global,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Global => "global",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Document {
    pub path: PathBuf,
    pub id: String,
    pub created_at: String,
    pub heading: String,
    pub body: String,
    pub kind: String,
    pub status: Option<String>,
    pub files: Vec<String>,
    pub topics: Vec<String>,
    pub related: Vec<String>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
    pub source_record: Option<String>,
    pub scope: Scope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredWorkspace {
    pub id: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistrationOutcome {
    Registered { id: String },
    AlreadyCurrent { id: String },
    Moved { id: String, previous: PathBuf },
    Copied { id: String },
}

#[derive(Debug)]
struct RegistryLock {
    path: PathBuf,
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
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
    let plan = root.join("plan.md");
    if !plan.exists() {
        fs::write(&plan, PLAN)
            .map_err(|error| format!("cannot write {}: {error}", plan.display()))?;
    }
    let later = root.join("later.md");
    if !later.exists() {
        fs::write(&later, LATER)
            .map_err(|error| format!("cannot write {}: {error}", later.display()))?;
    }
    Ok(root)
}

pub fn register_workspace(repo: &Path) -> Result<RegistrationOutcome, String> {
    let config = repo.join(".tincan/config.toml");
    validate_config(&config)?;
    let directory = personal_tincan_root()?.join("projects");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let _lock = lock_registry(&directory)?;
    let (mut id, _) = ensure_workspace_id(&config)?;
    let existing_entry = directory.join(format!("{id}.path"));
    let previous = read_workspace_entry(&existing_entry)?;
    let outcome = match previous {
        Some(previous) if previous == repo => {
            RegistrationOutcome::AlreadyCurrent { id: id.clone() }
        }
        Some(previous) if workspace_at_path_has_id(&previous, &id) => {
            id = replace_workspace_id(&config)?;
            RegistrationOutcome::Copied { id: id.clone() }
        }
        Some(previous) => RegistrationOutcome::Moved {
            id: id.clone(),
            previous,
        },
        None => RegistrationOutcome::Registered { id: id.clone() },
    };
    write_workspace_entry(&directory, &id, repo)?;
    remove_other_registrations_for_path(&directory, &id, repo)?;
    Ok(outcome)
}

fn write_workspace_entry(directory: &Path, id: &str, repo: &Path) -> Result<(), String> {
    let entry = directory.join(format!("{id}.path"));
    if read_workspace_entry(&entry)?.as_deref() != Some(repo) {
        atomic_write(&entry, encode_registry_path(repo).as_bytes())
            .map_err(|error| format!("cannot register workspace {}: {error}", repo.display()))?;
    }
    Ok(())
}

fn lock_registry(directory: &Path) -> Result<RegistryLock, String> {
    let path = directory.join("registry.lock");
    for attempt in 0..2 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(error) = writeln!(file, "{}", std::process::id()) {
                    let _ = fs::remove_file(&path);
                    return Err(format!(
                        "cannot write project registry lock {}: {error}",
                        path.display()
                    ));
                }
                return Ok(RegistryLock { path });
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::AlreadyExists
                    && attempt == 0
                    && registry_lock_is_stale(&path) =>
            {
                fs::remove_file(&path).map_err(|error| {
                    format!("cannot remove stale project registry lock: {error}")
                })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(
                    "another Tincan process is updating the project registry; retry shortly"
                        .to_string(),
                );
            }
            Err(error) => {
                return Err(format!(
                    "cannot lock project registry {}: {error}",
                    directory.display()
                ));
            }
        }
    }
    unreachable!()
}

fn registry_lock_is_stale(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age > std::time::Duration::from_secs(60))
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
    let temporary = path.with_extension(format!("tmp-{}", Uuid::now_v7()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    if let Err(error) = file.write_all(content).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    drop(file);
    if let Err(error) = replace_file(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, destination: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, destination)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, destination: &Path) -> Result<(), std::io::Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;

    if !destination.exists() {
        return fs::rename(temporary, destination);
    }
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let temporary = temporary
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            temporary.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn remove_other_registrations_for_path(
    directory: &Path,
    current_id: &str,
    repo: &Path,
) -> Result<(), String> {
    for workspace in registered_workspaces_with_problems_in(directory)?.0 {
        if workspace.id != current_id && workspace.path == repo {
            fs::remove_file(directory.join(format!("{}.path", workspace.id))).map_err(|error| {
                format!(
                    "cannot replace stale registration for {}: {error}",
                    repo.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn registered_workspaces_with_problems()
-> Result<(Vec<RegisteredWorkspace>, Vec<String>), String> {
    registered_workspaces_with_problems_in(&personal_tincan_root()?.join("projects"))
}

fn registered_workspaces_with_problems_in(
    directory: &Path,
) -> Result<(Vec<RegisteredWorkspace>, Vec<String>), String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), Vec::new()));
        }
        Err(error) => return Err(format!("cannot read {}: {error}", directory.display())),
    };
    let mut workspaces = Vec::new();
    let mut problems = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read workspace registry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("path") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if Uuid::parse_str(id).is_err() {
            continue;
        }
        match read_workspace_entry(&path) {
            Ok(Some(registered_path)) => workspaces.push(RegisteredWorkspace {
                id: id.to_string(),
                path: registered_path,
            }),
            Ok(None) => problems.push(format!("{} is empty", path.display())),
            Err(error) => problems.push(error),
        }
    }
    workspaces.sort_by(|left, right| left.id.cmp(&right.id));
    Ok((workspaces, problems))
}

pub fn registered_workspace_is_available(workspace: &RegisteredWorkspace) -> bool {
    let config = workspace.path.join(".tincan/config.toml");
    fs::read_to_string(config)
        .ok()
        .and_then(|content| config_string(&content, "workspace_id"))
        .is_some_and(|id| id == workspace.id)
}

pub fn remove_registered_workspace(id: &str) -> Result<Option<PathBuf>, String> {
    let directory = personal_tincan_root()?.join("projects");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let _lock = lock_registry(&directory)?;
    let entry = directory.join(format!("{id}.path"));
    let Some(registered_path) = read_workspace_entry(&entry)? else {
        return Ok(None);
    };
    fs::remove_file(&entry)
        .map_err(|error| format!("cannot remove {}: {error}", entry.display()))?;
    Ok(Some(registered_path))
}

fn read_workspace_id(config: &Path) -> Result<Option<String>, String> {
    let content = fs::read_to_string(config)
        .map_err(|error| format!("cannot read {}: {error}", config.display()))?;
    let Some(value) = config_string(&content, "workspace_id") else {
        return Ok(None);
    };
    Uuid::parse_str(&value).map_err(|_| {
        format!(
            "{} contains an invalid workspace_id: {value}",
            config.display()
        )
    })?;
    Ok(Some(value))
}

fn ensure_workspace_id(config: &Path) -> Result<(String, bool), String> {
    let mut content = fs::read_to_string(config)
        .map_err(|error| format!("cannot read {}: {error}", config.display()))?;
    if let Some(value) = config_string(&content, "workspace_id") {
        Uuid::parse_str(&value).map_err(|_| {
            format!(
                "{} contains an invalid workspace_id: {value}",
                config.display()
            )
        })?;
        return Ok((value, false));
    }
    let id = Uuid::now_v7().to_string();
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("workspace_id = \"{id}\"\n"));
    atomic_write(config, content.as_bytes())
        .map_err(|error| format!("cannot write {}: {error}", config.display()))?;
    Ok((id, true))
}

fn replace_workspace_id(config: &Path) -> Result<String, String> {
    let content = fs::read_to_string(config)
        .map_err(|error| format!("cannot read {}: {error}", config.display()))?;
    let id = Uuid::now_v7().to_string();
    let mut replaced = false;
    let mut output = String::new();
    for line in content.lines() {
        if line
            .split_once('=')
            .is_some_and(|(candidate, _)| candidate.trim() == "workspace_id")
        {
            output.push_str(&format!("workspace_id = \"{id}\"\n"));
            replaced = true;
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if !replaced {
        output.push_str(&format!("workspace_id = \"{id}\"\n"));
    }
    atomic_write(config, output.as_bytes())
        .map_err(|error| format!("cannot write {}: {error}", config.display()))?;
    Ok(id)
}

fn workspace_at_path_has_id(repo: &Path, id: &str) -> bool {
    read_workspace_id(&repo.join(".tincan/config.toml"))
        .ok()
        .flatten()
        .is_some_and(|candidate| candidate == id)
}

fn read_workspace_entry(path: &Path) -> Result<Option<PathBuf>, String> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    if content.is_empty() {
        return Ok(None);
    }
    decode_registry_path(&content)
        .map(Some)
        .map_err(|error| format!("invalid registry entry {}: {error}", path.display()))
}

fn encode_registry_path(path: &Path) -> String {
    let bytes = os_str_bytes(path.as_os_str());
    let mut output = String::from("path-v1\n");
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output.push('\n');
    output
}

fn decode_registry_path(content: &[u8]) -> Result<PathBuf, String> {
    if let Some(hex) = content.strip_prefix(b"path-v1\n") {
        let hex = hex.strip_suffix(b"\n").unwrap_or(hex);
        if hex.len() % 2 != 0 || !hex.iter().all(u8::is_ascii_hexdigit) {
            return Err("invalid path-v1 encoding".to_string());
        }
        let bytes = hex
            .chunks_exact(2)
            .map(|pair| {
                let text = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(text, 16).unwrap()
            })
            .collect::<Vec<_>>();
        return os_string_from_bytes(bytes).map(PathBuf::from);
    }
    let legacy =
        std::str::from_utf8(content).map_err(|_| "legacy path is not valid UTF-8".to_string())?;
    Ok(PathBuf::from(legacy.strip_suffix('\n').unwrap_or(legacy)))
}

#[cfg(unix)]
fn os_str_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}

#[cfg(unix)]
fn os_string_from_bytes(value: Vec<u8>) -> Result<OsString, String> {
    use std::os::unix::ffi::OsStringExt;
    Ok(OsString::from_vec(value))
}

#[cfg(windows)]
fn os_str_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    value.encode_wide().flat_map(u16::to_le_bytes).collect()
}

#[cfg(windows)]
fn os_string_from_bytes(value: Vec<u8>) -> Result<OsString, String> {
    use std::os::windows::ffi::OsStringExt;
    if value.len() % 2 != 0 {
        return Err("Windows path encoding has an odd byte count".to_string());
    }
    let wide = value
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    Ok(OsString::from_wide(&wide))
}

fn config_string(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        if candidate.trim() != key {
            return None;
        }
        let value = value.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(str::to_string)
    })
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

pub fn remove_git_excluded(path: &Path, pattern: &str) -> Result<bool, String> {
    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    if !existing.lines().any(|line| line.trim() == pattern) {
        return Ok(false);
    }
    let mut updated = existing
        .lines()
        .filter(|line| line.trim() != pattern)
        .collect::<Vec<_>>()
        .join("\n");
    if !updated.is_empty() {
        updated.push('\n');
    }
    fs::write(path, updated)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(true)
}

pub fn require(repo: &Path) -> Result<PathBuf, String> {
    let root = open(repo)?;
    for directory in DIRECTORIES {
        fs::create_dir_all(root.join(directory))
            .map_err(|error| format!("cannot create .tincan/{directory}: {error}"))?;
    }
    let plan = root.join("plan.md");
    if !plan.exists() {
        fs::write(&plan, PLAN)
            .map_err(|error| format!("cannot write {}: {error}", plan.display()))?;
    }
    let later = root.join("later.md");
    if !later.exists() {
        fs::write(&later, LATER)
            .map_err(|error| format!("cannot write {}: {error}", later.display()))?;
    }
    Ok(root)
}

fn open(repo: &Path) -> Result<PathBuf, String> {
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
    Ok(root)
}

pub fn read_plan(repo: &Path) -> Result<(PathBuf, String), String> {
    let path = open(repo)?.join("plan.md");
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok((path, content))
}

pub struct RememberUpdate {
    pub path: PathBuf,
    pub added: bool,
}

pub fn read_later(repo: &Path) -> Result<(PathBuf, String, usize), String> {
    let path = open(repo)?.join("later.md");
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => LATER.to_string(),
        Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
    };
    let count = content
        .lines()
        .filter(|line| line.starts_with("- "))
        .count();
    Ok((path, content, count))
}

pub fn remember(repo: &Path, text: &str) -> Result<RememberUpdate, String> {
    let root = require(repo)?;
    let path = root.join("later.md");
    let item = text.trim();
    if item.is_empty() {
        return Err("remember requires a non-empty item".to_string());
    }
    if item.contains(['\n', '\r']) {
        return Err("remember items must fit on one line".to_string());
    }
    let (_, existing, _) = read_later(repo)?;
    if existing
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .any(|value| value.trim() == item)
    {
        return Ok(RememberUpdate { path, added: false });
    }
    let mut content = existing
        .lines()
        .filter(|line| line.trim() != "<!-- none -->")
        .collect::<Vec<_>>()
        .join("\n");
    content.truncate(content.trim_end().len());
    content.push_str("\n\n- ");
    content.push_str(item);
    content.push('\n');
    fs::write(&path, content)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(RememberUpdate { path, added: true })
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
    pub planned: &'a [String],
    pub questions: &'a [String],
    pub next: &'a [String],
}

struct JournalRender<'a> {
    current: JournalSections<'a>,
    legacy_decisions: &'a [String],
    legacy_learnings: &'a [String],
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
    let legacy_decisions = markdown_bullets(&existing, "## Decisions");
    let legacy_learnings = markdown_bullets(&existing, "## Learnings");
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
    for item in clean_bullets(sections.planned) {
        if !planned_items.contains(&item) {
            planned_items.push(item);
            added += 1;
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

    let rendered = JournalRender {
        current: JournalSections {
            done: &done_items,
            planned: &planned_items,
            questions: &question_items,
            next: &next_items,
        },
        legacy_decisions: &legacy_decisions,
        legacy_learnings: &legacy_learnings,
    };
    let content = render_journal(date, &original_created_at, rendered);
    fs::write(&path, content)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(JournalUpdate { path, added })
}

pub fn latest_journal(repo: &Path) -> Result<Option<(PathBuf, String)>, String> {
    let root = open(repo)?;
    let directory = root.join("journal");
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("cannot read {}: {error}", directory.display())),
    };
    let mut paths = entries
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

fn render_journal(date: &str, created_at: &str, sections: JournalRender<'_>) -> String {
    let mut output = format!(
        "---\nid: {}\ntype: \"journal\"\ncreated_at: {}\n---\n\n# {date}\n",
        crate::util::yaml_string(&format!("journal-{date}")),
        crate::util::yaml_string(created_at),
    );
    let current = sections.current;
    let mut rendered_sections = vec![("Done", current.done)];
    if !sections.legacy_decisions.is_empty() {
        rendered_sections.push(("Decisions", sections.legacy_decisions));
    }
    if !sections.legacy_learnings.is_empty() {
        rendered_sections.push(("Learnings", sections.legacy_learnings));
    }
    rendered_sections.extend([
        ("Planned", current.planned),
        ("Open questions", current.questions),
        ("Next", current.next),
    ]);
    for (heading, values) in rendered_sections {
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
    let root = open(repo)?;
    let mut documents = Vec::new();
    for directory in DIRECTORIES {
        let directory = root.join(directory);
        if directory.is_dir() {
            collect_documents(&directory, Scope::Project, &mut documents)?;
        }
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

pub fn scan_global() -> Result<Vec<Document>, String> {
    let directory = global_learnings_directory()?;
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut documents = Vec::new();
    collect_documents(&directory, Scope::Global, &mut documents)?;
    if let Some(document) = documents
        .iter()
        .find(|document| document.kind != "learning")
    {
        return Err(format!(
            "invalid global record in {}: only learnings are supported",
            document.path.display()
        ));
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(documents)
}

pub fn write_global_learning(
    source: &Document,
    id: &str,
    created_at: &str,
    source_workspace: &str,
    body: &str,
) -> Result<PathBuf, String> {
    if source.kind != "learning" || source.scope != Scope::Project {
        return Err("only a project learning can be lifted".to_string());
    }
    let directory = global_learnings_directory()?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    let path = directory.join(format!("{id}.md"));
    let content = render_global_learning(source, id, created_at, source_workspace, body);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                format!("global learning already exists: {}", path.display())
            } else {
                format!("cannot write {}: {error}", path.display())
            }
        })?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    Ok(path)
}

pub fn validate_global_learning_body(body: &str) -> Result<(), String> {
    let body = body.trim();
    if body.starts_with("---") {
        return Err(
            "global learning draft must contain Markdown body only; remove YAML frontmatter"
                .to_string(),
        );
    }
    let first = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default();
    if first
        .strip_prefix("# ")
        .is_none_or(|heading| heading.trim().is_empty())
    {
        return Err("global learning draft must start with a non-empty H1 heading".to_string());
    }
    if body.lines().filter(|line| line.starts_with("# ")).count() != 1 {
        return Err("global learning draft must contain exactly one H1 heading".to_string());
    }
    Ok(())
}

fn collect_documents(path: &Path, scope: Scope, output: &mut Vec<Document>) -> Result<(), String> {
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
        let mut document = parse_document(file_path, text)?;
        document.scope = scope;
        output.push(document);
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
    let created_at = scalar(&text, "created_at").ok_or_else(|| {
        format!(
            "invalid frontmatter in {}: created_at is required",
            path.display()
        )
    })?;
    if matches!(kind.as_str(), "decision" | "learning") {
        uuid::Uuid::parse_str(&id).map_err(|_| {
            format!(
                "invalid frontmatter in {}: decision and learning ids must be UUIDs",
                path.display()
            )
        })?;
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
        if let Some(source_record) = scalar(&text, "source_record") {
            if uuid::Uuid::parse_str(&source_record).is_err() {
                return Err(format!(
                    "invalid frontmatter in {}: source_record must be a UUID",
                    path.display()
                ));
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
        created_at,
        heading,
        body,
        kind,
        status,
        files: yaml_list(&text, "files"),
        topics: yaml_list(&text, "topics"),
        related: yaml_list(&text, "related"),
        supersedes: yaml_list(&text, "supersedes"),
        superseded_by: yaml_list(&text, "superseded_by"),
        source_record: scalar(&text, "source_record"),
        scope: Scope::Project,
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
        decisions.push(document.clone());
    }
    Ok(decisions)
}

fn global_learnings_directory() -> Result<PathBuf, String> {
    Ok(personal_tincan_root()?.join("global").join("learnings"))
}

fn personal_tincan_root() -> Result<PathBuf, String> {
    let explicit = nonempty_env_path("TINCAN_HOME");
    #[cfg(test)]
    if explicit.is_none() {
        return Ok(std::env::temp_dir().join(format!("tincan-unit-tests-{}", std::process::id())));
    }
    let user_profile = nonempty_env_path("USERPROFILE");
    let home = nonempty_env_path("HOME");
    personal_tincan_root_from(explicit, user_profile, home).ok_or_else(|| {
        "cannot locate global Tincan storage; set TINCAN_HOME, USERPROFILE, or HOME".to_string()
    })
}

fn nonempty_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn personal_tincan_root_from(
    explicit: Option<PathBuf>,
    user_profile: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Option<PathBuf> {
    explicit.or_else(|| user_profile.or(home).map(|path| path.join(".tincan")))
}

fn render_global_learning(
    source: &Document,
    id: &str,
    created_at: &str,
    source_workspace: &str,
    body: &str,
) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str(&format!("id: {}\n", crate::util::yaml_string(id)));
    output.push_str("type: \"learning\"\n");
    output.push_str("scope: \"global\"\n");
    output.push_str(&format!(
        "created_at: {}\n",
        crate::util::yaml_string(created_at)
    ));
    output.push_str(&format!(
        "source_workspace: {}\n",
        crate::util::yaml_string(source_workspace)
    ));
    output.push_str(&format!(
        "source_record: {}\n",
        crate::util::yaml_string(&source.id)
    ));
    output.push_str("files:\n");
    output.push_str("topics:\n");
    for topic in &source.topics {
        output.push_str(&format!("  - {}\n", crate::util::yaml_string(topic)));
    }
    output.push_str("related:\n");
    output.push_str("supersedes:\n");
    output.push_str("superseded_by:\n");
    output.push_str("---\n\n");
    output.push_str(body.trim());
    output.push('\n');
    output
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
        assert_eq!(document.scope, Scope::Project);
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

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("target/\n"));
        assert!(content.contains("/.tincan/\n"));
        assert_eq!(content.matches(".tincan/").count(), 1);

        assert!(remove_git_excluded(&path, "/.tincan/").unwrap());
        assert!(!remove_git_excluded(&path, "/.tincan/").unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), "target/\n");
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
        assert!(!root.join("AGENT_GUIDE.md").exists());
        assert_eq!(fs::read_to_string(root.join("plan.md")).unwrap(), PLAN);
        assert_eq!(fs::read_to_string(root.join("later.md")).unwrap(), LATER);
        for directory in DIRECTORIES {
            assert!(root.join(directory).is_dir());
        }

        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn chooses_explicit_or_home_relative_personal_storage() {
        let explicit = PathBuf::from("custom-tincan");
        let profile = PathBuf::from("profile");
        let home = PathBuf::from("home");

        assert_eq!(
            personal_tincan_root_from(
                Some(explicit.clone()),
                Some(profile.clone()),
                Some(home.clone())
            ),
            Some(explicit)
        );
        assert_eq!(
            personal_tincan_root_from(None, Some(profile.clone()), Some(home.clone())),
            Some(profile.join(".tincan"))
        );
        assert_eq!(
            personal_tincan_root_from(None, None, Some(home.clone())),
            Some(home.join(".tincan"))
        );
        assert_eq!(personal_tincan_root_from(None, None, None), None);
    }

    #[test]
    fn validates_prepared_global_learning_markdown() {
        validate_global_learning_body(
            "# Prefer presentation-boundary normalization\n\nKeep canonical paths internally.",
        )
        .unwrap();
        assert!(
            validate_global_learning_body("Body without a heading")
                .unwrap_err()
                .contains("must start with")
        );
        assert!(
            validate_global_learning_body("---\nid: owned\n---\n\n# Heading")
                .unwrap_err()
                .contains("remove YAML frontmatter")
        );
        assert!(
            validate_global_learning_body("# First\n\n# Second")
                .unwrap_err()
                .contains("exactly one H1")
        );
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
        assert!(!content.contains("## Decisions"));
        assert!(!content.contains("## Learnings"));
        assert!(content.contains("## Planned\n\n- Add a plan"));
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn preserves_legacy_decision_and_learning_sections_without_creating_new_ones() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-legacy-journal-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        let root = initialize(&repo).unwrap();
        let path = root.join("journal/2026-08-05.md");
        fs::write(
            &path,
            "---\nid: \"journal-2026-08-05\"\ntype: \"journal\"\ncreated_at: \"2026-08-05T09:00:00+02:00\"\n---\n\n# 2026-08-05\n\n## Done\n\n<!-- none -->\n\n## Decisions\n\n- Keep Markdown canonical\n\n## Learnings\n\n- Nested repositories need workspace-relative paths\n\n## Planned\n\n<!-- none -->\n\n## Open questions\n\n<!-- none -->\n\n## Next\n\n<!-- none -->\n",
        )
        .unwrap();

        update_journal(
            &repo,
            "2026-08-05",
            "2026-08-05T10:00:00+02:00",
            JournalSections {
                done: &[],
                planned: &["Add a plan".into()],
                questions: &[],
                next: &[],
            },
        )
        .unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("## Decisions\n\n- Keep Markdown canonical"));
        assert!(
            content.contains("## Learnings\n\n- Nested repositories need workspace-relative paths")
        );
        assert!(content.contains("## Planned\n\n- Add a plan"));
        fs::remove_dir_all(repo).unwrap();
    }

    #[test]
    fn registry_path_encoding_round_trips_without_trimming() {
        let path = PathBuf::from(" leading and trailing ");
        let encoded = encode_registry_path(&path);
        assert_eq!(decode_registry_path(encoded.as_bytes()).unwrap(), path);
        assert_eq!(
            decode_registry_path(b" legacy path \n").unwrap(),
            PathBuf::from(" legacy path ")
        );
    }

    #[test]
    fn registry_lock_rejects_a_concurrent_writer() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("tincan-registry-lock-{unique}"));
        fs::create_dir_all(&directory).unwrap();
        let first = lock_registry(&directory).unwrap();
        assert!(
            lock_registry(&directory)
                .unwrap_err()
                .contains("retry shortly")
        );
        drop(first);
        assert!(lock_registry(&directory).is_ok());
        fs::remove_dir_all(directory).unwrap();
    }
}
