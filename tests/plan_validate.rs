use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("speq"))
}

/// Copy a fixture directory to a temp directory for testing
fn setup_fixture(tmp: &TempDir, fixture_name: &str) {
    let fixture_path = Path::new("tests/fixtures/plan_validate").join(fixture_name);
    let dest_path = tmp.path().join("specs/_plans").join(fixture_name);
    copy_dir_recursive(&fixture_path, &dest_path).unwrap();
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Copy the target specs a fixture's deltas merge against into `specs/`.
///
/// A no-op when the fixture carries no `_targets/<fixture_name>/` directory,
/// so every existing fixture without a target spec keeps passing unmodified.
fn setup_target_specs(tmp: &TempDir, fixture_name: &str) {
    let targets_path = Path::new("tests/fixtures/plan_validate/_targets").join(fixture_name);
    if !targets_path.exists() {
        return;
    }
    let dest_path = tmp.path().join("specs");
    copy_dir_recursive(&targets_path, &dest_path).unwrap();
}

mod plan_exists {
    use super::*;

    #[test]
    fn validates_plan_with_plan_md() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "valid-plan");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "valid-plan"])
            .assert()
            .success();
    }

    #[test]
    fn fails_when_plan_directory_missing() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("specs/_plans")).unwrap();

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "nonexistent"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("Plan not found"));
    }

    #[test]
    fn fails_when_plan_md_missing() {
        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("specs/_plans/incomplete");
        fs::create_dir_all(&plan_dir).unwrap();

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "incomplete"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("plan.md not found"));
    }

    #[test]
    fn checks_deltas_when_plan_md_missing() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "missing-plan-md");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "missing-plan-md"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("plan.md not found"))
            .stdout(predicate::str::contains("DELTA:NEW not closed"));
    }

    #[test]
    fn resolves_the_plan_from_inside_the_plan_directory() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "valid-plan");

        cmd()
            .current_dir(tmp.path().join("specs/_plans/valid-plan"))
            .args(["plan", "validate", "valid-plan"])
            .assert()
            .success();
    }
}

mod delta_markers {
    use super::*;

    #[test]
    fn passes_with_matched_delta_markers() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "valid-delta");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "valid-delta"])
            .assert()
            .success();
    }

    #[test]
    fn fails_with_unclosed_delta_new_marker() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "unclosed-new");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "unclosed-new"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("DELTA:NEW not closed"));
    }

    #[test]
    fn fails_with_unclosed_delta_changed_marker() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "unclosed-changed");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "unclosed-changed"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("DELTA:CHANGED not closed"));
    }

    #[test]
    fn fails_with_unclosed_delta_removed_marker() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "unclosed-removed");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "unclosed-removed"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("DELTA:REMOVED not closed"));
    }

    #[test]
    fn reports_line_number_for_unclosed_marker() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "unclosed-new");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "unclosed-new"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("line 11"));
    }
}

mod spec_validation {
    use super::*;

    #[test]
    fn passes_with_correctly_formatted_deltas() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "valid-delta");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "valid-delta"])
            .assert()
            .success()
            .stdout(predicate::str::contains("validation passed"));
    }

    #[test]
    fn fails_with_malformed_step_formatting() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "malformed-steps");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "malformed-steps"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("missing a GIVEN step"));
    }

    #[test]
    fn fails_with_steps_missing_emphasized_keywords() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "missing-emphasis");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "missing-emphasis"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("missing a GIVEN step"));
    }

    #[test]
    fn warns_on_lowercase_step_keywords() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "lowercase-steps");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "lowercase-steps"])
            .assert()
            .success()
            .stdout(predicate::str::contains("should be uppercase"));
    }

    #[test]
    fn warns_on_lowercase_rfc_keywords() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "lowercase-rfc");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "lowercase-rfc"])
            .assert()
            .success()
            .stdout(predicate::str::contains("should be uppercase"));
    }

    #[test]
    fn passes_plan_without_delta_specs() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "no-deltas");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "no-deltas"])
            .assert()
            .success();
    }

    #[test]
    fn fails_for_nonexistent_plan() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("specs/_plans")).unwrap();

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "ghost-plan"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("Plan not found"));
    }

    #[test]
    fn includes_standard_spec_validation_errors() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "incomplete-spec");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "incomplete-spec"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("ERROR"));
    }
}

mod decision_log {
    use super::*;

    #[test]
    fn plan_with_valid_decision_log_passes() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "with-decisions");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "with-decisions"])
            .assert()
            .success();
    }

    #[test]
    fn plan_without_decision_log_passes() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "valid-plan");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "valid-plan"])
            .assert()
            .success();
    }

    #[test]
    fn plan_decision_log_no_sections_fails() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "decisions-no-sections");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "decisions-no-sections"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("section"));
    }

    #[test]
    fn plan_decision_log_bad_promote_warns() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "decisions-bad-promote");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "decisions-bad-promote"])
            .assert()
            .success()
            .stdout(predicate::str::contains("maybe"));
    }

    #[test]
    fn plan_decision_log_bad_h1_fails() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "decisions-bad-h1");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "decisions-bad-h1"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("wrong-name"));
    }
}

mod delta_anchors {
    use super::*;

    #[test]
    fn passes_with_changed_prose_anchors() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "prose-changed");
        setup_target_specs(&tmp, "prose-changed");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "prose-changed"])
            .assert()
            .success()
            .stdout(predicate::str::contains("unmarked").not());
    }

    #[test]
    fn warns_about_unmarked_edits_to_an_existing_feature() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "unmarked-edit");
        setup_target_specs(&tmp, "unmarked-edit");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "unmarked-edit"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "test/feature/spec.md: unmarked `## Background` differs from the recorded spec",
            ))
            .stdout(predicate::str::contains(
                "unmarked `### Scenario: Unmarked addition` does not exist in the recorded spec",
            ))
            .stdout(predicate::str::contains("Scenario: Existing one").not());
    }

    #[test]
    fn fails_with_new_or_removed_on_prose_anchor() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "prose-bad-kind");
        setup_target_specs(&tmp, "prose-bad-kind");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "prose-bad-kind"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("DELTA:NEW"))
            .stdout(predicate::str::contains("DELTA:REMOVED"))
            .stdout(predicate::str::contains("## Background"))
            .stdout(predicate::str::contains("# Feature: <name>"));
    }

    #[test]
    fn fails_with_unrecognized_anchor() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "prose-no-anchor");
        setup_target_specs(&tmp, "prose-no-anchor");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "prose-no-anchor"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("### Scenario:"))
            .stdout(predicate::str::contains("## Background"))
            .stdout(predicate::str::contains("# Feature:"));
    }

    #[test]
    fn fails_with_nested_delta_markers() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "nested-markers");
        setup_target_specs(&tmp, "nested-markers");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "nested-markers"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("Malformed delta marker"));
    }

    #[test]
    fn skips_anchor_checks_for_new_feature() {
        let tmp = TempDir::new().unwrap();
        setup_fixture(&tmp, "prose-new-feature");
        setup_target_specs(&tmp, "prose-new-feature");

        cmd()
            .current_dir(tmp.path())
            .args(["plan", "validate", "prose-new-feature"])
            .assert()
            .success();
    }
}
