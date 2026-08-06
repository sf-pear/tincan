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

    fs::remove_dir_all(workspace).unwrap();
}
