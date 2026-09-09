use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tincan() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tincan"));
    let test_name = std::thread::current()
        .name()
        .unwrap_or("unnamed-test")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    command.env(
        "TINCAN_HOME",
        std::env::temp_dir().join(format!(
            "tincan-cli-tests-{}-{test_name}",
            std::process::id()
        )),
    );
    command
}

fn workspace_id(workspace: &std::path::Path) -> String {
    fs::read_to_string(workspace.join(".tincan/config.toml"))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("workspace_id = \"")?.strip_suffix('"'))
        .unwrap()
        .to_string()
}

#[test]
fn help_succeeds_and_explains_the_main_workflow() {
    let output = tincan().arg("--help").output().unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("tincan decide"));
    assert!(stdout.contains("tincan learn"));
    assert!(stdout.contains("SKILL INSTALL"));
}

#[test]
fn invalid_input_fails_and_writes_the_error_to_stderr() {
    let output = tincan().arg("not-a-command").output().unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("tincan: unknown command: not-a-command"));
    assert!(stderr.contains("tincan <COMMAND> [OPTIONS]"));
}

#[test]
fn init_explains_existing_workspaces_and_registration_state() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-init-status-{unique}"));
    let workspace = root.join("workspace");
    let tincan_home = root.join("home");
    fs::create_dir_all(&workspace).unwrap();

    let first = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&workspace)
        .output()
        .unwrap();
    let first_stdout = String::from_utf8(first.stdout).unwrap();
    assert!(first.status.success());
    assert!(first_stdout.contains("Initialized Tincan at"));
    assert!(first_stdout.contains("Added this workspace to your Tincan projects"));
    assert!(first_stdout.contains("Run tincan projects"));

    let second = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&workspace)
        .output()
        .unwrap();
    let second_stdout = String::from_utf8(second.stdout).unwrap();
    assert!(second.status.success());
    assert!(second_stdout.contains("Tincan is already initialized at"));
    assert!(second_stdout.contains("This workspace is already in your Tincan projects"));
    assert!(second_stdout.contains("Run tincan projects"));

    let unregister = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .args(["projects", "unregister"])
        .arg(&workspace)
        .output()
        .unwrap();
    assert!(unregister.status.success());

    let reregister = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&workspace)
        .output()
        .unwrap();
    let reregister_stdout = String::from_utf8(reregister.stdout).unwrap();
    assert!(reregister.status.success());
    assert!(reregister_stdout.contains("Tincan is already initialized at"));
    assert!(reregister_stdout.contains("Added this workspace to your Tincan projects"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn keeps_a_small_deduplicated_later_shelf_in_resume_context() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = std::env::temp_dir().join(format!("tincan-cli-later-{unique}"));
    fs::create_dir_all(&workspace).unwrap();
    assert!(
        tincan()
            .arg("init")
            .arg(&workspace)
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        workspace.join(".tincan/later.md"),
        "# Later\n\nNotes here stay editable.\n\n<!-- none -->\n",
    )
    .unwrap();

    for _ in 0..2 {
        let output = tincan()
            .current_dir(&workspace)
            .args(["remember", "Explore a shorter review default"])
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    let later = tincan()
        .current_dir(&workspace)
        .arg("later")
        .output()
        .unwrap();
    let later_stdout = String::from_utf8(later.stdout).unwrap();
    assert_eq!(
        later_stdout
            .matches("Explore a shorter review default")
            .count(),
        1
    );
    assert!(later_stdout.contains("Notes here stay editable."));

    let resume = tincan()
        .current_dir(&workspace)
        .arg("resume")
        .output()
        .unwrap();
    let resume_stdout = String::from_utf8(resume.stdout).unwrap();
    assert!(resume_stdout.contains("Later:"));
    assert!(resume_stdout.contains("Explore a shorter review default"));
    assert!(resume_stdout.find("Plan:") < resume_stdout.find("Later:"));
    assert!(resume_stdout.find("Later:") < resume_stdout.find("No journal entries yet."));

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn resume_does_not_require_access_to_the_personal_project_registry() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = std::env::temp_dir().join(format!("tincan-cli-local-resume-{unique}"));
    let tincan_home = std::env::temp_dir().join(format!("tincan-cli-blocked-home-{unique}"));
    fs::create_dir_all(&workspace).unwrap();
    assert!(
        tincan()
            .arg("init")
            .arg(&workspace)
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(&tincan_home, "not a directory").unwrap();

    let resume = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&workspace)
        .arg("resume")
        .output()
        .unwrap();

    assert!(
        resume.status.success(),
        "resume failed: {}",
        String::from_utf8_lossy(&resume.stderr)
    );
    assert!(String::from_utf8(resume.stdout).unwrap().contains("Plan:"));
    assert!(resume.stderr.is_empty());

    fs::remove_dir_all(workspace).unwrap();
    fs::remove_file(tincan_home).unwrap();
}

#[test]
fn review_replaces_summary_and_writes_selected_context_safely() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = std::env::temp_dir().join(format!("tincan-cli-review-{unique}"));
    fs::create_dir_all(&workspace).unwrap();
    let init = tincan().arg("init").arg(&workspace).output().unwrap();
    assert!(init.status.success());
    fs::write(
        workspace.join(".tincan/journal/2025-08-05.md"),
        "---\nid: \"journal-2025-08-05\"\ntype: \"journal\"\ncreated_at: \"2025-08-05T09:00:00+02:00\"\n---\n\n# 2025-08-05\n\n## Done\n\n- Shipped useful review context\n\n## Planned\n\n<!-- none -->\n\n## Open questions\n\n<!-- none -->\n\n## Next\n\n<!-- none -->\n",
    )
    .unwrap();
    fs::write(
        workspace.join(
            ".tincan/decisions/019fd6d9-1ff8-7082-9f86-2b7d89712a57.md",
        ),
        "---\nid: \"019fd6d9-1ff8-7082-9f86-2b7d89712a57\"\ntype: \"decision\"\nstatus: \"active\"\ncreated_at: \"2025-09-10T09:00:00+02:00\"\nfiles:\ntopics:\nrelated:\nsupersedes:\nsuperseded_by:\n---\n\n# Keep review deterministic\n",
    )
    .unwrap();

    let overview = tincan()
        .current_dir(&workspace)
        .arg("review")
        .output()
        .unwrap();
    assert!(
        overview.status.success(),
        "{}",
        String::from_utf8_lossy(&overview.stderr)
    );
    let overview_stdout = String::from_utf8(overview.stdout).unwrap();
    assert!(overview_stdout.contains("Review history: 2025-08-05 through 2025-09-10"));
    assert!(overview_stdout.contains("2025              1           1           0"));

    let destination = workspace.join("review-2025-q3.md");
    let selected = tincan()
        .current_dir(&workspace)
        .args(["review", "--quarter", "2025-Q3", "--output"])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(selected.status.success());
    let content = fs::read_to_string(&destination).unwrap();
    assert!(content.contains("# Tincan review: 2025 Q3"));
    assert!(content.contains("Shipped useful review context"));
    assert!(content.contains("Keep review deterministic"));
    assert!(!content.contains("<!-- none -->"));

    let protected = tincan()
        .current_dir(&workspace)
        .args(["review", "--year", "2025", "--output"])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(!protected.status.success());
    assert!(
        String::from_utf8(protected.stderr)
            .unwrap()
            .contains("use --force to replace it")
    );

    let replaced = tincan()
        .current_dir(&workspace)
        .args(["review", "--year", "2025", "--output"])
        .arg(&destination)
        .arg("--force")
        .output()
        .unwrap();
    assert!(replaced.status.success());
    assert!(
        fs::read_to_string(&destination)
            .unwrap()
            .contains("# Tincan review: 2025")
    );

    let removed = tincan()
        .current_dir(&workspace)
        .arg("summary")
        .output()
        .unwrap();
    assert!(!removed.status.success());
    assert!(
        String::from_utf8(removed.stderr)
            .unwrap()
            .contains("summary was replaced by review")
    );
    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn cross_project_review_requires_init_to_register_a_moved_workspace() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-projects-{unique}"));
    let tincan_home = root.join("personal-tincan");
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    for project in [&first, &second] {
        let output = tincan()
            .env("TINCAN_HOME", &tincan_home)
            .arg("init")
            .arg(project)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::write(
        first.join(".tincan/journal/2025-08-05.md"),
        "---\nid: \"journal-2025-08-05\"\ntype: \"journal\"\ncreated_at: \"2025-08-05T09:00:00Z\"\n---\n\n# 2025-08-05\n\n## Done\n\n- Worked on the first project\n",
    )
    .unwrap();
    fs::write(
        second.join(".tincan/journal/2025-08-06.md"),
        "---\nid: \"journal-2025-08-06\"\ntype: \"journal\"\ncreated_at: \"2025-08-06T09:00:00Z\"\n---\n\n# 2025-08-06\n\n## Done\n\n- Worked on the second project\n",
    )
    .unwrap();
    fs::remove_file(second.join(".tincan/later.md")).unwrap();
    let stale_id = workspace_id(&second);

    let moved = root.join("first-moved");
    fs::rename(&first, &moved).unwrap();
    let local_plan = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&moved)
        .arg("plan")
        .output()
        .unwrap();
    assert!(local_plan.status.success());

    let stale_review = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&root)
        .args(["review", "--year", "2025", "--all-projects"])
        .output()
        .unwrap();
    let stale_content = String::from_utf8(stale_review.stdout).unwrap();
    assert!(stale_content.contains("## Unavailable projects"));
    assert!(stale_content.contains("first"));
    assert!(!stale_content.contains("Worked on the first project"));

    let register_move = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&moved)
        .output()
        .unwrap();
    assert!(register_move.status.success());
    assert!(
        String::from_utf8(register_move.stdout)
            .unwrap()
            .contains("Updated this workspace in your Tincan projects")
    );

    let review = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&root)
        .args(["review", "--year", "2025", "--all-projects"])
        .output()
        .unwrap();
    assert!(
        review.status.success(),
        "{}",
        String::from_utf8_lossy(&review.stderr)
    );
    let content = String::from_utf8(review.stdout).unwrap();
    assert!(content.contains("# Tincan review: 2025 — all projects"));
    assert!(content.contains("## first-moved"));
    assert!(content.contains("Worked on the first project"));
    assert!(content.contains("## second"));
    assert!(content.contains("Worked on the second project"));
    assert!(!content.contains("Unavailable projects"));
    assert!(!second.join(".tincan/later.md").exists());

    fs::remove_dir_all(&second).unwrap();
    let stale_review = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&root)
        .args(["review", "--year", "2025", "--all-projects"])
        .output()
        .unwrap();
    let stale_content = String::from_utf8(stale_review.stdout).unwrap();
    assert!(stale_content.contains("## Unavailable projects"));
    assert!(stale_content.contains("second"));

    let removed = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .args(["projects", "unregister", &stale_id])
        .output()
        .unwrap();
    assert!(removed.status.success());
    assert!(
        !tincan_home
            .join("projects")
            .join(format!("{stale_id}.path"))
            .exists()
    );

    let unregistered_current = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .args(["projects", "unregister"])
        .arg(&moved)
        .output()
        .unwrap();
    assert!(unregistered_current.status.success());
    assert_eq!(
        fs::read_dir(tincan_home.join("projects")).unwrap().count(),
        0
    );
    let plan_after_unregister = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&moved)
        .arg("plan")
        .output()
        .unwrap();
    assert!(plan_after_unregister.status.success());
    assert_eq!(
        fs::read_dir(tincan_home.join("projects")).unwrap().count(),
        0
    );
    let reregistered = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&moved)
        .output()
        .unwrap();
    assert!(reregistered.status.success());
    assert_eq!(
        fs::read_dir(tincan_home.join("projects")).unwrap().count(),
        1
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cross_project_review_reports_broken_projects_and_registry_entries() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-partial-review-{unique}"));
    let tincan_home = root.join("personal-tincan");
    let healthy = root.join("healthy");
    let broken = root.join("broken");
    for project in [&healthy, &broken] {
        fs::create_dir_all(project).unwrap();
        let output = tincan()
            .env("TINCAN_HOME", &tincan_home)
            .arg("init")
            .arg(project)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    fs::write(
        healthy.join(".tincan/journal/2025-08-05.md"),
        "---\nid: \"journal-2025-08-05\"\ntype: \"journal\"\ncreated_at: \"2025-08-05T09:00:00Z\"\n---\n\n# 2025-08-05\n\n## Done\n\n- Healthy project remains visible\n",
    )
    .unwrap();
    fs::write(
        broken.join(".tincan/journal/not-a-date.md"),
        "---\nid: \"journal-not-a-date\"\ntype: \"journal\"\ncreated_at: \"not-a-date\"\n---\n\n# not-a-date\n",
    )
    .unwrap();
    fs::write(
        tincan_home
            .join("projects")
            .join("019fd6d9-1ff8-7082-9f86-2b7d89712a57.path"),
        "path-v1\nnot-hex\n",
    )
    .unwrap();

    let review = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&root)
        .args(["review", "--year", "2025", "--all-projects"])
        .output()
        .unwrap();
    assert!(review.status.success());
    let content = String::from_utf8(review.stdout).unwrap();
    assert!(content.contains("Healthy project remains visible"));
    assert!(content.contains("## Unavailable projects"));
    assert!(content.contains("Invalid registry entry"));
    assert!(content.contains("invalid review date"));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn duplicate_live_workspace_ids_are_not_silently_relocated() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-duplicate-id-{unique}"));
    let tincan_home = root.join("personal-tincan");
    let original = root.join("original");
    let copied = root.join("copied");
    fs::create_dir_all(&original).unwrap();
    let init = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&original)
        .output()
        .unwrap();
    assert!(init.status.success());
    fs::create_dir_all(copied.join(".tincan")).unwrap();
    fs::copy(
        original.join(".tincan/config.toml"),
        copied.join(".tincan/config.toml"),
    )
    .unwrap();
    fs::copy(
        original.join(".tincan/plan.md"),
        copied.join(".tincan/plan.md"),
    )
    .unwrap();

    let local_plan = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&copied)
        .arg("plan")
        .output()
        .unwrap();
    assert!(local_plan.status.success());
    let projects = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("projects")
        .output()
        .unwrap();
    let listed = String::from_utf8(projects.stdout).unwrap();
    assert!(listed.contains("original"));
    assert!(!listed.contains("\\copied") && !listed.contains("/copied"));

    let reinitialize_copy = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&copied)
        .output()
        .unwrap();
    assert!(reinitialize_copy.status.success());
    assert!(
        String::from_utf8(reinitialize_copy.stdout)
            .unwrap()
            .contains("Added this copy as a separate workspace")
    );
    assert_ne!(workspace_id(&original), workspace_id(&copied));
    assert_eq!(
        fs::read_dir(tincan_home.join("projects")).unwrap().count(),
        2
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn path_unregistration_does_not_assign_an_id_to_a_legacy_workspace() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-legacy-unregister-{unique}"));
    let tincan_home = root.join("personal-tincan");
    fs::create_dir_all(root.join(".tincan")).unwrap();
    let config = root.join(".tincan/config.toml");
    let original = "# Tincan workspace configuration\nversion = 2\nstorage = \"markdown\"\n";
    fs::write(&config, original).unwrap();

    let result = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .args(["projects", "unregister"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(config).unwrap(), original);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn one_workspace_operates_across_two_nested_git_repositories() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let workspace = std::env::temp_dir().join(format!("tincan-cli-workspace-{unique}"));
    let api = workspace.join("api");
    let web = workspace.join("web");
    for repo in [&api, &web] {
        fs::create_dir_all(repo.join("src")).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(repo)
                .status()
                .unwrap()
                .success()
        );
    }

    let init = tincan().arg("init").arg(&workspace).output().unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(workspace.join(".tincan/plan.md").is_file());

    let decision = tincan()
        .current_dir(&api)
        .args([
            "decide",
            "Keep API changes backward compatible",
            "--file",
            "api/src/new.rs",
        ])
        .output()
        .unwrap();
    assert!(
        decision.status.success(),
        "{}",
        String::from_utf8_lossy(&decision.stderr)
    );
    fs::write(api.join("src/new.rs"), "pub fn new() {}\n").unwrap();
    fs::write(web.join("src/new.ts"), "export const value = 1;\n").unwrap();

    let changes = tincan().current_dir(&web).arg("changes").output().unwrap();
    assert!(
        changes.status.success(),
        "{}",
        String::from_utf8_lossy(&changes.stderr)
    );
    let stdout = String::from_utf8(changes.stdout).unwrap();
    assert!(stdout.contains("api/src/new.rs  decision: Keep API changes backward compatible"));
    assert!(stdout.contains("web/src/new.ts  no records"));

    let plan = tincan().current_dir(&api).arg("plan").output().unwrap();
    assert!(plan.status.success());
    assert!(String::from_utf8(plan.stdout).unwrap().contains("# Plan"));

    let resumed = tincan().current_dir(&api).arg("resume").output().unwrap();
    assert!(resumed.status.success());
    let resumed_stdout = String::from_utf8(resumed.stdout).unwrap();
    assert!(resumed_stdout.contains("Plan:"));
    assert!(resumed_stdout.contains("# Plan"));
    assert!(resumed_stdout.contains("No journal entries yet."));

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn lifts_and_retrieves_a_global_learning_inside_or_outside_a_workspace() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tincan-cli-global-{unique}"));
    let workspace = root.join("workspace");
    let outside = root.join("outside");
    let tincan_home = root.join("personal-tincan");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let init = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .arg("init")
        .arg(&workspace)
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let learned = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&workspace)
        .args([
            "learn",
            "HomeTime must normalize its Windows paths before printing",
            "--topic",
            "windows",
            "--topic",
            "paths",
            "--evidence",
            "Observed in CLI output",
        ])
        .output()
        .unwrap();
    assert!(
        learned.status.success(),
        "{}",
        String::from_utf8_lossy(&learned.stderr)
    );
    let learned_stdout = String::from_utf8(learned.stdout).unwrap();
    let source_id = learned_stdout
        .lines()
        .find_map(|line| line.strip_prefix("Record ID: "))
        .unwrap();

    let draft = root.join("global-learning.md");
    fs::write(
        &draft,
        "# Normalize platform-specific paths at presentation boundaries\n\nKeep canonical paths internally and normalize only when displaying them.\n",
    )
    .unwrap();

    let lifted = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&workspace)
        .arg("lift")
        .arg(source_id)
        .arg("--from")
        .arg(&draft)
        .output()
        .unwrap();
    assert!(
        lifted.status.success(),
        "{}",
        String::from_utf8_lossy(&lifted.stderr)
    );
    let lifted_stdout = String::from_utf8(lifted.stdout).unwrap();
    let global_id = lifted_stdout
        .lines()
        .find_map(|line| line.strip_prefix("Record ID: "))
        .unwrap();
    let global_path = tincan_home
        .join("global/learnings")
        .join(format!("{global_id}.md"));
    let global_content = fs::read_to_string(&global_path).unwrap();
    assert!(global_content.contains("scope: \"global\""));
    assert!(global_content.contains(&format!("source_record: \"{source_id}\"")));
    assert!(global_content.contains("topics:\n  - \"windows\"\n  - \"paths\""));
    assert!(!global_content.contains("files:\n  -"));
    assert!(global_content.contains("# Normalize platform-specific paths"));
    assert!(!global_content.contains("HomeTime"));

    fs::remove_dir_all(tincan_home.join("projects")).unwrap();
    fs::write(tincan_home.join("projects"), "registry unavailable").unwrap();

    let project_search = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&workspace)
        .args(["search", "windows"])
        .output()
        .unwrap();
    assert!(project_search.status.success());
    let project_results = String::from_utf8(project_search.stdout).unwrap();
    assert!(project_results.contains("[project learning]"));
    assert!(project_results.contains("[global learning]"));

    let global_search = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&outside)
        .args(["search", "windows"])
        .output()
        .unwrap();
    assert!(global_search.status.success());
    let global_results = String::from_utf8(global_search.stdout).unwrap();
    assert!(!global_results.contains("[project learning]"));
    assert!(global_results.contains("[global learning]"));

    let shown = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&outside)
        .args(["show", global_id])
        .output()
        .unwrap();
    assert!(shown.status.success());
    assert!(
        String::from_utf8(shown.stdout)
            .unwrap()
            .contains("# Normalize platform-specific paths at presentation boundaries")
    );

    let duplicate = tincan()
        .env("TINCAN_HOME", &tincan_home)
        .current_dir(&workspace)
        .arg("lift")
        .arg(source_id)
        .arg("--from")
        .arg(&draft)
        .output()
        .unwrap();
    assert!(duplicate.status.success());
    assert!(
        String::from_utf8(duplicate.stdout)
            .unwrap()
            .contains("Learning is already global")
    );

    fs::remove_dir_all(root).unwrap();
}
