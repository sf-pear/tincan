use std::process::Command;

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
