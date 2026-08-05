use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Snapshot {
    pub root: PathBuf,
    pub branch: String,
    pub status: String,
    pub changed_files: Vec<String>,
    pub recent_commits: String,
    pub diff_stat: String,
}

pub fn repository_root(path: &Path) -> Result<PathBuf, String> {
    let output = run(path, &["rev-parse", "--show-toplevel"])?;
    let root = output.trim();
    if root.is_empty() {
        return Err(format!("{} is not inside a Git repository", path.display()));
    }
    Ok(PathBuf::from(root))
}

pub fn snapshot(path: &Path) -> Result<Snapshot, String> {
    let root = repository_root(path)?;
    Ok(Snapshot {
        branch: run(&root, &["branch", "--show-current"])?
            .trim()
            .to_string(),
        status: run(&root, &["status", "--short"])?,
        changed_files: changed_files(&root)?,
        recent_commits: run(
            &root,
            &["log", "-10", "--pretty=format:%h%x09%s", "--no-decorate"],
        )
        .unwrap_or_else(|_| "No commits yet".to_string()),
        diff_stat: run(&root, &["diff", "--stat", "HEAD"])
            .or_else(|_| run(&root, &["diff", "--stat"]))
            .unwrap_or_default(),
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
