use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn repository_root(path: &Path) -> Result<Option<PathBuf>, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;
    if !output.status.success() {
        if path.ancestors().any(|parent| parent.join(".git").exists()) {
            return Err(format!(
                "git rev-parse --show-toplevel failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        return Ok(None);
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok((!root.is_empty()).then(|| PathBuf::from(root)))
}

pub fn branch(path: &Path) -> Result<String, String> {
    let Some(root) = repository_root(path)? else {
        return Ok(String::new());
    };
    Ok(run(&root, &["branch", "--show-current"])?
        .trim()
        .to_string())
}

pub fn protect_workspace(workspace: &Path) -> Result<Option<bool>, String> {
    let Some(repo) = repository_root(workspace)? else {
        return Ok(None);
    };
    let relative = workspace.strip_prefix(&repo).map_err(|_| {
        format!(
            "{} is outside Git repository {}",
            workspace.display(),
            repo.display()
        )
    })?;
    let prefix = slash(relative);
    let tincan = if prefix.is_empty() {
        ".tincan".to_string()
    } else {
        format!("{prefix}/.tincan")
    };
    let tracked = run(&repo, &["ls-files", "--", &tincan])?;
    if !tracked.is_empty() {
        return Err(format!(
            "{tincan} already contains tracked files, so Tincan cannot guarantee private storage:\n{}\nremove them from the Git index before running `tincan init`",
            tracked
                .lines()
                .map(|file| format!("  {file}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    let exclude = exclude_path(&repo)?;
    let pattern = format!("/{tincan}/");
    let changed = crate::store::ensure_git_excluded(&exclude, &pattern)?;
    verify_ignored(&repo, &format!("{tincan}/config.toml"))?;
    Ok(Some(changed))
}

pub fn workspace_changed_files(workspace: &Path) -> Result<Option<Vec<String>>, String> {
    if let Some(repo) = repository_root(workspace)? {
        let relative = slash(workspace.strip_prefix(&repo).unwrap_or(Path::new("")));
        let files = changed_files(&repo)?
            .into_iter()
            .filter_map(|file| strip_workspace_prefix(&file, &relative))
            .collect::<BTreeSet<_>>();
        return Ok(Some(files.into_iter().collect()));
    }

    let repositories = descendant_repositories(workspace)?;
    if repositories.is_empty() {
        return Ok(None);
    }
    let mut files = BTreeSet::new();
    for repo in repositories {
        let prefix = slash(repo.strip_prefix(workspace).unwrap_or(&repo));
        for file in changed_files(&repo)? {
            files.insert(if prefix.is_empty() {
                file
            } else {
                format!("{prefix}/{file}")
            });
        }
    }
    Ok(Some(files.into_iter().collect()))
}

fn descendant_repositories(workspace: &Path) -> Result<Vec<PathBuf>, String> {
    let mut repos = Vec::new();
    let mut pending = vec![workspace.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name();
            if matches!(
                name.to_str(),
                Some(".git" | ".tincan" | "target" | "node_modules" | ".venv")
            ) {
                continue;
            }
            if path.join(".git").exists() {
                repos.push(path);
            } else {
                pending.push(path);
            }
        }
    }
    repos.sort();
    Ok(repos)
}

fn strip_workspace_prefix(file: &str, prefix: &str) -> Option<String> {
    if prefix.is_empty() {
        return Some(file.to_string());
    }
    file.strip_prefix(&(prefix.to_string() + "/"))
        .map(str::to_string)
}

fn changed_files(repo: &Path) -> Result<Vec<String>, String> {
    let mut files = BTreeSet::new();
    let diff = run(repo, &["diff", "--name-only", "HEAD"])
        .or_else(|_| run(repo, &["diff", "--name-only"]))?;
    files.extend(
        diff.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.replace('\\', "/")),
    );
    let untracked = run(repo, &["ls-files", "--others", "--exclude-standard"])?;
    files.extend(
        untracked
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.replace('\\', "/")),
    );
    Ok(files.into_iter().collect())
}

fn exclude_path(repo: &Path) -> Result<PathBuf, String> {
    let value = run(repo, &["rev-parse", "--git-path", "info/exclude"])?;
    let path = PathBuf::from(value.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

fn verify_ignored(repo: &Path, path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["check-ignore", "--quiet", "--no-index", "--", path])
        .current_dir(repo)
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    if output.status.code() == Some(1) {
        return Err(format!(
            "Git does not ignore {path}; refusing to initialize memory that may be committed. Review repository ignore rules and retry."
        ));
    }
    Err(format!(
        "git check-ignore failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn slash(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
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
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tincan-{name}-{unique}"))
    }

    #[test]
    fn aggregates_changes_from_nested_repositories() {
        let workspace = temp("multi-repo");
        for name in ["api", "web"] {
            let repo = workspace.join(name);
            fs::create_dir_all(repo.join("src")).unwrap();
            run(&repo, &["init", "--quiet"]).unwrap();
            fs::write(repo.join("src/new.txt"), name).unwrap();
        }
        assert_eq!(
            workspace_changed_files(&workspace).unwrap().unwrap(),
            vec!["api/src/new.txt", "web/src/new.txt"]
        );
        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn protects_a_workspace_below_a_git_root() {
        let repo = temp("nested-private");
        let workspace = repo.join("projects");
        fs::create_dir_all(&workspace).unwrap();
        run(&repo, &["init", "--quiet"]).unwrap();

        assert_eq!(protect_workspace(&workspace).unwrap(), Some(true));
        crate::store::initialize(&workspace).unwrap();
        let status = run(&repo, &["status", "--short", "--untracked-files=all"]).unwrap();
        assert!(
            status.is_empty(),
            "private memory appeared in status: {status}"
        );
        fs::remove_dir_all(repo).unwrap();
    }
}
