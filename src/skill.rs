use std::fs;
use std::path::{Path, PathBuf};

const SKILL: &str = include_str!("../skills/tincan/SKILL.md");
const OPENAI_METADATA: &str = include_str!("../skills/tincan/agents/openai.yaml");

pub enum InstallOutcome {
    Installed(PathBuf),
    AlreadyCurrent(PathBuf),
}

pub fn install(path: Option<&Path>, force: bool) -> Result<InstallOutcome, String> {
    let skills = match path {
        Some(path) => path.to_path_buf(),
        None => default_skills_directory()?,
    };
    let destination = skills.join("tincan");
    let skill_path = destination.join("SKILL.md");
    let metadata_path = destination.join("agents").join("openai.yaml");

    if skill_path.is_file()
        && metadata_path.is_file()
        && read(&skill_path)? == SKILL
        && read(&metadata_path)? == OPENAI_METADATA
    {
        return Ok(InstallOutcome::AlreadyCurrent(destination));
    }
    if destination.exists() && !force {
        return Err(format!(
            "{} already exists and differs from this Tincan version; rerun with --force to update Tincan-owned files",
            destination.display()
        ));
    }

    fs::create_dir_all(destination.join("agents")).map_err(|error| {
        format!(
            "cannot create skill directory {}: {error}",
            destination.display()
        )
    })?;
    fs::write(&skill_path, SKILL)
        .map_err(|error| format!("cannot write {}: {error}", skill_path.display()))?;
    fs::write(&metadata_path, OPENAI_METADATA)
        .map_err(|error| format!("cannot write {}: {error}", metadata_path.display()))?;
    Ok(InstallOutcome::Installed(destination))
}

fn default_skills_directory() -> Result<PathBuf, String> {
    if let Some(path) = nonempty_env("CODEX_HOME") {
        return Ok(PathBuf::from(path).join("skills"));
    }
    let home = nonempty_env("USERPROFILE")
        .or_else(|| nonempty_env("HOME"))
        .ok_or_else(|| {
            "cannot find the home directory; pass --path with the harness skills directory"
                .to_string()
        })?;
    Ok(PathBuf::from(home).join(".codex").join("skills"))
}

fn nonempty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn installs_idempotently_and_protects_different_content() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tincan-skill-install-{unique}"));

        assert!(matches!(
            install(Some(&root), false).unwrap(),
            InstallOutcome::Installed(_)
        ));
        assert_eq!(
            fs::read_to_string(root.join("tincan").join("SKILL.md")).unwrap(),
            SKILL
        );
        assert!(matches!(
            install(Some(&root), false).unwrap(),
            InstallOutcome::AlreadyCurrent(_)
        ));

        fs::write(root.join("tincan").join("SKILL.md"), "local edit").unwrap();
        assert!(install(Some(&root), false).is_err());
        assert!(install(Some(&root), true).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
}
