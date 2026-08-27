use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tincan() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tincan"))
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
        .args(["review", "quarter", "3", "--year", "2025", "--output"])
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
