use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

const PROJECT: &str = "tests/fixtures/feature_get";
const SPEC: &str = "tests/fixtures/feature_get/specs/demo/inline/spec.md";

fn cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("speq"))
}

mod formatted_output {
    use super::*;

    #[test]
    fn keeps_inline_code_spans() {
        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "demo/inline"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "connection strings like `exasol://host:port`.",
            ))
            .stdout(predicate::str::contains(
                "Given a connection string `exasol://db:8563`",
            ));
    }

    #[test]
    fn shows_description_and_background_separately() {
        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "demo/inline"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Inline Code\n\nThe driver SHALL accept connection strings like `exasol://host:port`.\n\n## Background",
            ))
            .stdout(predicate::str::contains(
                "  * Connection strings follow the format `exasol://host:port`.",
            ))
            .stdout(predicate::str::contains("  * Default port: `8563`"));
    }

    #[test]
    fn finds_scenario_whose_name_has_inline_code() {
        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "demo/inline/Parse `host:port`"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Then the driver SHALL read host `db` and port `8563`",
            ));
    }
}

mod raw_output {
    use super::*;

    #[test]
    fn prints_the_spec_file_verbatim() {
        let expected = fs::read_to_string(SPEC).unwrap();

        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "--raw", "demo/inline"])
            .assert()
            .success()
            .stdout(expected);
    }

    #[test]
    fn prints_only_the_scenario_section() {
        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "--raw", "demo/inline/Other scenario"])
            .assert()
            .success()
            .stdout(
                "### Scenario: Other scenario\n\n* *GIVEN* setup\n* *WHEN* action\n* *THEN* result SHALL happen\n",
            );
    }

    #[test]
    fn reports_a_missing_scenario() {
        cmd()
            .current_dir(PROJECT)
            .args(["feature", "get", "--raw", "demo/inline/Missing"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains(
                "Scenario 'Missing' not found in demo/inline",
            ));
    }
}

mod project_root {
    use super::*;

    #[test]
    fn resolves_specs_from_a_subdirectory() {
        cmd()
            .current_dir("tests/fixtures/feature_get/specs/demo/inline/nested")
            .args(["feature", "list"])
            .assert()
            .success()
            .stdout(predicate::str::contains("inline"));
    }

    #[test]
    fn fails_when_no_specs_directory_exists() {
        let tmp = TempDir::new().unwrap();

        cmd()
            .current_dir(tmp.path())
            .args(["feature", "list"])
            .assert()
            .code(1)
            .stdout(predicate::str::contains("No specs/ directory found in"));
    }
}
