use dialoguer::{Confirm, MultiSelect, theme::ColorfulTheme};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SKILL: &str = include_str!("../skills/tincan/SKILL.md");
const OPENAI_METADATA: &str = include_str!("../skills/tincan/agents/openai.yaml");
const PICKER_HELP: &str =
    "[↑↓ move, Space select/unselect, A toggle all/none, Enter continue, Esc cancel]";

#[derive(Debug)]
pub enum InstallOutcome {
    Installed(PathBuf),
    AlreadyCurrent(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillRoot {
    pub name: String,
    pub path: PathBuf,
}

pub fn detect_roots() -> Vec<SkillRoot> {
    let home = nonempty_env("USERPROFILE")
        .or_else(|| nonempty_env("HOME"))
        .map(PathBuf::from);
    let codex_home = nonempty_env("CODEX_HOME").map(PathBuf::from);
    detect_roots_from(home.as_deref(), codex_home.as_deref())
}

pub fn choose_interactively(roots: &[SkillRoot]) -> Result<Option<Vec<PathBuf>>, String> {
    let labels = selection_labels(roots);
    let defaults = default_selections(roots);
    let prompt = format!("Select user-wide Agent Skills destinations\n{PICKER_HELP}");
    let theme = ColorfulTheme::default();
    loop {
        let Some(indices) = MultiSelect::with_theme(&theme)
            .with_prompt(&prompt)
            .items(&labels)
            .defaults(&defaults)
            .interact_opt()
            .map_err(|error| format!("cannot read skill destination selection: {error}"))?
        else {
            return Ok(None);
        };
        let selected = selected_paths(roots, &indices);
        if selected.is_empty() {
            eprintln!("No destinations selected. Select at least one or press Escape to cancel.");
            continue;
        }
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt(format!(
                "Install Tincan in {} selected destination{}?",
                selected.len(),
                if selected.len() == 1 { "" } else { "s" }
            ))
            .default(false)
            .wait_for_newline(true)
            .interact()
            .map_err(|error| format!("cannot read skill installation confirmation: {error}"))?;
        return Ok(confirmed.then_some(selected));
    }
}

pub fn install_many(roots: &[PathBuf], force: bool) -> Result<Vec<InstallOutcome>, String> {
    let plans = roots
        .iter()
        .map(|root| install_plan(root, force))
        .collect::<Result<Vec<_>, _>>()?;
    plans.into_iter().map(apply_install_plan).collect()
}

fn install_plan(skills: &Path, force: bool) -> Result<InstallPlan, String> {
    let destination = skills.join("tincan");
    let skill_path = destination.join("SKILL.md");
    let metadata_path = destination.join("agents").join("openai.yaml");

    if skill_path.is_file()
        && metadata_path.is_file()
        && read(&skill_path)? == SKILL
        && read(&metadata_path)? == OPENAI_METADATA
    {
        return Ok(InstallPlan::Current(destination));
    }
    if destination.exists() && !force {
        return Err(format!(
            "{} already exists and differs from this Tincan version; rerun with --force to update Tincan-owned files",
            destination.display()
        ));
    }

    Ok(InstallPlan::Write(destination))
}

fn apply_install_plan(plan: InstallPlan) -> Result<InstallOutcome, String> {
    let destination = match plan {
        InstallPlan::Current(destination) => {
            return Ok(InstallOutcome::AlreadyCurrent(destination));
        }
        InstallPlan::Write(destination) => destination,
    };
    let skill_path = destination.join("SKILL.md");
    let metadata_path = destination.join("agents").join("openai.yaml");
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

enum InstallPlan {
    Current(PathBuf),
    Write(PathBuf),
}

fn detect_roots_from(home: Option<&Path>, codex_home: Option<&Path>) -> Vec<SkillRoot> {
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(codex_home) = codex_home {
        add_detected_root(
            &mut roots,
            &mut seen,
            "Codex",
            codex_home,
            &codex_home.join("skills"),
        );
    }

    if let Some(home) = home {
        for (name, directory) in [
            ("Agent Skills", ".agents"),
            ("Claude Code", ".claude"),
            ("Codex", ".codex"),
            ("Cursor", ".cursor"),
            ("Gemini CLI", ".gemini"),
        ] {
            let marker = home.join(directory);
            add_detected_root(&mut roots, &mut seen, name, &marker, &marker.join("skills"));
        }
        let opencode = home.join(".config").join("opencode");
        add_detected_root(
            &mut roots,
            &mut seen,
            "OpenCode",
            &opencode,
            &opencode.join("skills"),
        );
    }

    roots
}

fn add_detected_root(
    roots: &mut Vec<SkillRoot>,
    seen: &mut BTreeSet<String>,
    name: &str,
    marker: &Path,
    skills: &Path,
) {
    if !marker.is_dir() {
        return;
    }
    let mut key = skills.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        key.make_ascii_lowercase();
    }
    if seen.insert(key) {
        roots.push(SkillRoot {
            name: name.to_string(),
            path: skills.to_path_buf(),
        });
    }
}

fn selection_labels(roots: &[SkillRoot]) -> Vec<String> {
    roots
        .iter()
        .map(|root| {
            format!(
                "{}: {}",
                root.name,
                root.path.to_string_lossy().replace(['\n', '\r'], " ")
            )
        })
        .collect()
}

fn default_selections(roots: &[SkillRoot]) -> Vec<bool> {
    vec![true; roots.len()]
}

fn selected_paths(roots: &[SkillRoot], indices: &[usize]) -> Vec<PathBuf> {
    indices
        .iter()
        .filter_map(|index| roots.get(*index))
        .map(|root| root.path.clone())
        .collect()
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
            install_many(std::slice::from_ref(&root), false).unwrap()[0],
            InstallOutcome::Installed(ref path) if path == &root.join("tincan")
        ));
        assert_eq!(
            fs::read_to_string(root.join("tincan").join("SKILL.md")).unwrap(),
            SKILL
        );
        assert!(matches!(
            install_many(std::slice::from_ref(&root), false).unwrap()[0],
            InstallOutcome::AlreadyCurrent(ref path) if path == &root.join("tincan")
        ));

        fs::write(root.join("tincan").join("SKILL.md"), "local edit").unwrap();
        assert!(install_many(std::slice::from_ref(&root), false).is_err());
        assert!(install_many(std::slice::from_ref(&root), true).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn detects_only_harnesses_present_on_the_machine() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tincan-skill-detect-{unique}"));
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude")).unwrap();
        fs::create_dir_all(home.join(".cursor")).unwrap();

        let roots = detect_roots_from(Some(&home), None);
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|root| root.name == "Claude Code"));
        assert!(roots.iter().any(|root| root.name == "Cursor"));
        assert!(!roots.iter().any(|root| root.name.contains("Codex")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn renders_each_destination_on_one_line_and_maps_selections() {
        let roots = vec![
            SkillRoot {
                name: "First".to_string(),
                path: PathBuf::from("first/skills"),
            },
            SkillRoot {
                name: "Second".to_string(),
                path: PathBuf::from("second/skills"),
            },
        ];
        let labels = selection_labels(&roots);
        assert_eq!(labels.len(), 2);
        assert!(labels[0].contains("First: first/skills"));
        assert!(labels.iter().all(|label| !label.contains('\n')));
        let selected = selected_paths(&roots, &[1]);
        assert_eq!(selected, vec![PathBuf::from("second/skills")]);
    }

    #[test]
    fn selects_every_detected_destination_by_default() {
        let roots = vec![SkillRoot {
            name: "Only harness".to_string(),
            path: PathBuf::from("only/skills"),
        }];
        assert_eq!(default_selections(&roots), vec![true]);
    }

    #[test]
    fn picker_help_explains_every_available_action() {
        for instruction in [
            "↑↓ move",
            "Space select/unselect",
            "A toggle all/none",
            "Enter continue",
            "Esc cancel",
        ] {
            assert!(PICKER_HELP.contains(instruction));
        }
    }

    #[test]
    fn bundled_skill_offers_initialization_without_assuming_consent() {
        assert!(SKILL.contains("structured"));
        assert!(SKILL.contains("user-question"));
        assert!(SKILL.contains("Initialize Tincan"));
        assert!(SKILL.contains("Not now"));
        assert!(SKILL.contains("Never run `tincan init` without the user's explicit confirmation"));
    }

    #[test]
    fn validates_every_destination_before_writing_any() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tincan-skill-atomic-{unique}"));
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(second.join("tincan")).unwrap();
        fs::write(second.join("tincan").join("SKILL.md"), "owned elsewhere").unwrap();

        assert!(install_many(&[first.clone(), second], false).is_err());
        assert!(!first.join("tincan").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
