use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("speq"))
}

/// Copy a fixture spec into a temp project and return its validate output.
fn validate_fixture(fixture_name: &str) -> String {
    let tmp = TempDir::new().unwrap();
    let fixture_path = Path::new("tests/fixtures/and_step_limit")
        .join(fixture_name)
        .join("spec.md");
    let dest_dir = tmp.path().join("specs/test/feature");
    fs::create_dir_all(&dest_dir).unwrap();
    fs::copy(&fixture_path, dest_dir.join("spec.md")).unwrap();

    let output = cmd()
        .current_dir(tmp.path())
        .args(["feature", "validate", "test/feature"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8_lossy(&output).into_owned()
}

#[test]
fn warns_on_more_than_three_and_steps_after_then() {
    let stdout = validate_fixture("then-side");
    assert!(stdout.contains("4 AND steps after THEN"), "{stdout}");
    assert!(
        stdout.contains("Split it into separate scenarios"),
        "{stdout}"
    );
}

#[test]
fn and_steps_before_then_do_not_trigger_the_warning() {
    let stdout = validate_fixture("given-side");
    assert!(!stdout.contains("AND steps"), "{stdout}");
}
