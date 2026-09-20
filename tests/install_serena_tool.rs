//! Integration tests for `install.sh`'s `install_serena_tool` step and the
//! `serena` handling in `register_codex_mcp_servers`.
//!
//! These tests source `install.sh` and invoke the functions directly, with
//! `uv`, `serena`, and `codex` replaced by fake scripts on a restricted
//! `PATH` (a temp dir of fakes, plus `/usr/bin:/bin` only — never the real
//! `PATH`, so a real `uv`/`serena`/`codex` on the host can never leak in and
//! silently change the outcome).

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// Absolute path to the repository's `install.sh`.
fn install_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh")
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap_or_else(|e| panic!("chmod {}: {e}", path.display()));
}

/// A restricted PATH: `dir` first, then only `/usr/bin:/bin` — never the
/// real `PATH`.
fn restricted_path(dir: &Path) -> String {
    format!("{}:/usr/bin:/bin", dir.display())
}

/// Guard against a stale assumption: panics if `cmd` is resolvable on the
/// bare `/usr/bin:/bin` PATH, since that would invalidate any test that
/// relies on `cmd` being absent.
fn assert_absent_from_restricted_path(cmd: &str) {
    let output = Command::new("bash")
        .args(["-c", &format!("command -v {cmd}")])
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("run command -v");
    if output.status.success() {
        panic!(
            "'{cmd}' is unexpectedly resolvable on /usr/bin:/bin ({}); \
             this test's absence assumption is invalid on this machine",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
}

/// Fake `uv` that appends its arguments to a log file and exits with
/// `$FAKE_UV_EXIT` (default 0).
fn install_fake_uv(dir: &Path) -> PathBuf {
    let log = dir.join("uv.log");
    let script = format!(
        r#"#!/usr/bin/env bash
echo "$@" >> "{log}"
exit "${{FAKE_UV_EXIT:-0}}"
"#,
        log = log.display()
    );
    write_executable(&dir.join("uv"), &script);
    log
}

/// Fake `serena` that exists on PATH and exits 0 for anything.
fn install_fake_serena(dir: &Path) {
    write_executable(&dir.join("serena"), "#!/usr/bin/env bash\nexit 0\n");
}

/// Fake `codex` covering the shapes `register_codex_mcp_servers` uses:
/// - `mcp get serena` prints `$FAKE_CODEX_SERENA_GET_OUTPUT` and exits 0 when
///   `$FAKE_CODEX_SERENA_REGISTERED=1`, otherwise exits 1 ("not registered").
/// - `mcp get context7` always exits 1 ("not registered").
/// - `mcp add`/`mcp remove` append their arguments to a log file and exit
///   with `$FAKE_CODEX_ADD_EXIT`/`$FAKE_CODEX_REMOVE_EXIT` (default 0).
fn install_fake_codex(dir: &Path) -> PathBuf {
    let log = dir.join("codex.log");
    let script = format!(
        r#"#!/usr/bin/env bash
if [[ "$1" == "mcp" ]]; then
    case "$2" in
        get)
            case "$3" in
                serena)
                    if [[ "${{FAKE_CODEX_SERENA_REGISTERED:-0}}" == "1" ]]; then
                        echo "${{FAKE_CODEX_SERENA_GET_OUTPUT:-command: serena}}"
                        exit 0
                    fi
                    exit 1
                    ;;
                context7)
                    exit 1
                    ;;
            esac
            exit 1
            ;;
        add)
            shift 2
            echo "add $*" >> "{log}"
            exit "${{FAKE_CODEX_ADD_EXIT:-0}}"
            ;;
        remove)
            shift 2
            echo "remove $*" >> "{log}"
            exit "${{FAKE_CODEX_REMOVE_EXIT:-0}}"
            ;;
    esac
fi
exit 1
"#,
        log = log.display()
    );
    write_executable(&dir.join("codex"), &script);
    log
}

fn run_function(function: &str, path: &str, envs: &[(&str, &str)]) -> Output {
    let command = format!("source {} && {function}", install_script().display());
    let mut cmd = Command::new("bash");
    cmd.args(["-c", &command]).env("PATH", path);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run install.sh function")
}

fn read_log(log: &Path) -> String {
    fs::read_to_string(log).unwrap_or_default()
}

// ---------------------------------------------------------------------
// install_serena_tool
// ---------------------------------------------------------------------

#[test]
fn installer_installs_serena_with_uv_tool() {
    assert_absent_from_restricted_path("serena");

    let fake_bin = TempDir::new().unwrap();
    let uv_log = install_fake_uv(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    let output = run_function("install_serena_tool", &path, &[]);

    assert!(
        output.status.success(),
        "install_serena_tool failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = read_log(&uv_log);
    assert_eq!(log.trim(), "tool install -p 3.13 serena-agent");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Installed Serena CLI"),
        "expected success message, got: {stdout}"
    );
}

#[test]
fn installer_skips_serena_when_on_path() {
    let fake_bin = TempDir::new().unwrap();
    install_fake_serena(fake_bin.path());
    // Deliberately no fake uv: if install_serena_tool tried to use uv, this
    // would fail loudly (uv not found) instead of silently succeeding.
    let path = restricted_path(fake_bin.path());

    let output = run_function("install_serena_tool", &path, &[]);

    assert!(
        output.status.success(),
        "install_serena_tool failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("already installed"),
        "expected an already-installed message, got: {stdout}"
    );
}

#[test]
fn installer_warns_when_uv_absent() {
    assert_absent_from_restricted_path("serena");
    assert_absent_from_restricted_path("uv");

    let fake_bin = TempDir::new().unwrap();
    let path = restricted_path(fake_bin.path());

    let output = run_function("install_serena_tool", &path, &[]);

    assert!(
        output.status.success(),
        "install_serena_tool should not fail when uv is absent: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("uv not found"),
        "expected a uv-not-found warning, got: {stdout}"
    );
    assert!(
        stdout.contains("curl -LsSf https://astral.sh/uv/install.sh | sh"),
        "expected the uv install instructions, got: {stdout}"
    );
    assert!(
        stdout.contains("uv tool install -p 3.13 serena-agent"),
        "expected the manual serena install command, got: {stdout}"
    );
}

#[test]
fn installer_continues_when_uv_tool_install_fails() {
    assert_absent_from_restricted_path("serena");

    let fake_bin = TempDir::new().unwrap();
    install_fake_uv(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    let output = run_function("install_serena_tool", &path, &[("FAKE_UV_EXIT", "1")]);

    assert!(
        output.status.success(),
        "install_serena_tool must be non-fatal when uv tool install fails: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Serena CLI installation failed"),
        "expected a failure warning, got: {stdout}"
    );
    assert!(
        stdout.contains("uv tool install -p 3.13 serena-agent"),
        "expected the manual install command, got: {stdout}"
    );
}

// ---------------------------------------------------------------------
// register_codex_mcp_servers — serena registration by command
// ---------------------------------------------------------------------

#[test]
fn installer_registers_codex_serena_by_command() {
    let fake_bin = TempDir::new().unwrap();
    let codex_log = install_fake_codex(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    // Default FAKE_CODEX_SERENA_REGISTERED=0 => `codex mcp get serena` fails
    // ("not registered").
    let output = run_function("register_codex_mcp_servers", &path, &[]);

    assert!(
        output.status.success(),
        "register_codex_mcp_servers failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = read_log(&codex_log);
    assert!(
        log.contains("add serena -- serena start-mcp-server --project-from-cwd --context=codex"),
        "expected the by-command add invocation, got log: {log}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Registered Codex MCP server: serena"),
        "expected a registration success message, got: {stdout}"
    );
}

#[test]
fn installer_keeps_by_command_codex_registration() {
    let fake_bin = TempDir::new().unwrap();
    let codex_log = install_fake_codex(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    let output = run_function(
        "register_codex_mcp_servers",
        &path,
        &[
            ("FAKE_CODEX_SERENA_REGISTERED", "1"),
            ("FAKE_CODEX_SERENA_GET_OUTPUT", "command: serena"),
        ],
    );

    assert!(
        output.status.success(),
        "register_codex_mcp_servers failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = read_log(&codex_log);
    assert!(
        !log.contains("remove serena") && !log.contains("add serena"),
        "an already-correct by-command registration must not be touched, got log: {log}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("already registered: serena"),
        "expected an already-registered message, got: {stdout}"
    );
}

#[test]
fn installer_replaces_git_sourced_codex_registration() {
    let fake_bin = TempDir::new().unwrap();
    let codex_log = install_fake_codex(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    let output = run_function(
        "register_codex_mcp_servers",
        &path,
        &[
            ("FAKE_CODEX_SERENA_REGISTERED", "1"),
            ("FAKE_CODEX_SERENA_GET_OUTPUT", "command: uvx"),
        ],
    );

    assert!(
        output.status.success(),
        "register_codex_mcp_servers failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = read_log(&codex_log);
    assert!(
        log.contains("remove serena"),
        "expected the stale registration to be removed, got log: {log}"
    );
    assert!(
        log.contains("add serena -- serena start-mcp-server --project-from-cwd --context=codex"),
        "expected the by-command add invocation after removal, got log: {log}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Replacing") && stdout.contains("serena"),
        "expected a replacing message, got: {stdout}"
    );
    assert!(
        stdout.contains("Registered Codex MCP server: serena"),
        "expected a registration success message after replacing, got: {stdout}"
    );
}

#[test]
fn installer_warns_when_codex_remove_fails() {
    let fake_bin = TempDir::new().unwrap();
    let codex_log = install_fake_codex(fake_bin.path());
    let path = restricted_path(fake_bin.path());

    let output = run_function(
        "register_codex_mcp_servers",
        &path,
        &[
            ("FAKE_CODEX_SERENA_REGISTERED", "1"),
            ("FAKE_CODEX_SERENA_GET_OUTPUT", "command: uvx"),
            ("FAKE_CODEX_REMOVE_EXIT", "1"),
        ],
    );

    assert!(
        output.status.success(),
        "register_codex_mcp_servers failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = read_log(&codex_log);
    assert!(
        !log.contains("add serena"),
        "add must be skipped when remove fails, got log: {log}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Failed to remove existing Codex MCP server registration: serena"),
        "expected a remove-failure warning, got: {stdout}"
    );
    assert!(
        stdout.contains("codex mcp remove serena"),
        "expected the manual remove command, got: {stdout}"
    );
    assert!(
        stdout.contains("serena start-mcp-server --project-from-cwd --context=codex"),
        "expected the manual add command, got: {stdout}"
    );
}
