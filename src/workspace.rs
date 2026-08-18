use std::path::{Path, PathBuf};

pub fn find(path: &Path) -> Result<PathBuf, String> {
    let start = canonical_directory(path)?;
    find_from(&start).ok_or_else(|| {
        format!(
            "{} is not inside a Tincan workspace; run `tincan init {}`",
            start.display(),
            start.display()
        )
    })
}

pub fn find_optional(path: &Path) -> Result<Option<PathBuf>, String> {
    let start = canonical_directory(path)?;
    Ok(find_from(&start))
}

pub fn target(path: &Path) -> Result<PathBuf, String> {
    canonical_directory(path)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("cannot access {}: {error}", path.display()))?;
    if !path.is_dir() {
        return Err(format!("{} is not a directory", path.display()));
    }
    Ok(path)
}

fn find_from(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join(".tincan/config.toml").is_file())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn finds_nearest_workspace_from_a_nested_directory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tincan-workspace-{unique}"));
        let nested = root.join("repo/src");
        fs::create_dir_all(root.join(".tincan")).unwrap();
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(".tincan/config.toml"), "version = 2\n").unwrap();

        assert_eq!(find(&nested).unwrap(), root.canonicalize().unwrap());
        assert_eq!(
            find_optional(&nested).unwrap(),
            Some(root.canonicalize().unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn optionally_reports_that_no_workspace_exists() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tincan-no-workspace-{unique}"));
        fs::create_dir_all(&root).unwrap();

        assert_eq!(find_optional(&root).unwrap(), None);

        fs::remove_dir_all(root).unwrap();
    }
}
