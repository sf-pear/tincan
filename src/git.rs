use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Snapshot {
    pub root: PathBuf,
    pub branch: String,
    pub changed_files: Vec<String>,
}

pub fn repository_root(path: &Path) -> Result<PathBuf, String> {
    let output = run(path, &["rev-parse", "--show-toplevel"])?;
    let root = output.trim();
    if root.is_empty() {
        return Err(format!("{} is not inside a Git repository", path.display()));
    }
    Ok(PathBuf::from(root))
}

pub fn exclude_path(path: &Path) -> Result<PathBuf, String> {
    let root = repository_root(path)?;
    let value = run(&root, &["rev-parse", "--git-path", "info/exclude"])?;
    let path = PathBuf::from(value.trim());
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(root.join(path))
    }
}

pub fn require_tincan_untracked(path: &Path) -> Result<(), String> {
    let root = repository_root(path)?;
    let tracked = run(&root, &["ls-files", "--", ".tincan"])?;
    if tracked.is_empty() {
        return Ok(());
    }

    Err(format!(
        ".tincan already contains tracked files, so Tincan cannot guarantee private storage:\n{}\nremove them from the Git index before running `tincan init`",
        tracked
            .lines()
            .map(|file| format!("  {file}"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

pub fn verify_tincan_ignored(path: &Path) -> Result<(), String> {
    let root = repository_root(path)?;
    let output = Command::new("git")
        .args([
            "check-ignore",
            "--quiet",
            "--no-index",
            "--",
            ".tincan/config.toml",
        ])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;

    if output.status.success() {
        return Ok(());
    }
    if output.status.code() == Some(1) {
        return Err(
            "Git does not ignore .tincan/config.toml; refusing to initialize memory that may be committed. Review repository ignore rules that re-include .tincan and retry."
                .to_string(),
        );
    }

    let message = String::from_utf8_lossy(&output.stderr);
    Err(format!("git check-ignore failed: {}", message.trim()))
}

pub fn snapshot(path: &Path) -> Result<Snapshot, String> {
    let root = repository_root(path)?;
    Ok(Snapshot {
        branch: run(&root, &["branch", "--show-current"])?
            .trim()
            .to_string(),
        changed_files: changed_files(&root)?,
        root,
    })
}

pub fn changed_files(path: &Path) -> Result<Vec<String>, String> {
    let root = repository_root(path)?;
    let mut files = BTreeSet::new();
    let diff = run(&root, &["diff", "--name-only", "HEAD"])
        .or_else(|_| run(&root, &["diff", "--name-only"]))?;
    for line in diff.lines().map(str::trim).filter(|line| !line.is_empty()) {
        files.insert(line.replace('\\', "/"));
    }

    let untracked = run(&root, &["ls-files", "--others", "--exclude-standard"])?;
    for line in untracked
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        files.insert(line.replace('\\', "/"));
    }
    Ok(files.into_iter().collect())
}

fn run(path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string())
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {} failed: {}", args.join(" "), message.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn private_tincan_storage_is_ignored_and_must_be_untracked() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!("tincan-private-git-{unique}"));
        fs::create_dir_all(&repo).unwrap();
        run(&repo, &["init", "--quiet"]).unwrap();

        let exclude = exclude_path(&repo).unwrap();
        assert!(store::ensure_git_excluded(&exclude).unwrap());
        verify_tincan_ignored(&repo).unwrap();
        store::initialize(&repo).unwrap();

        let status = run(&repo, &["status", "--short", "--untracked-files=all"]).unwrap();
        assert!(
            status.is_empty(),
            "private memory appeared in status: {status}"
        );

        let repository_ignore = repo.join(".gitignore");
        fs::write(&repository_ignore, "!/.tincan/\n!/.tincan/config.toml\n").unwrap();
        let error = verify_tincan_ignored(&repo).unwrap_err();
        assert!(error.contains("does not ignore"));
        fs::remove_file(repository_ignore).unwrap();

        run(&repo, &["add", "--force", ".tincan/config.toml"]).unwrap();
        let error = require_tincan_untracked(&repo).unwrap_err();
        assert!(error.contains(".tincan/config.toml"));

        fs::remove_dir_all(repo).unwrap();
    }
}
