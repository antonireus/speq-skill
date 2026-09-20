use std::fs;
use std::path::Path;
use std::process::Command;

fn read_mcp_template(filename: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir)
        .join("scripts/plugin")
        .join(filename);
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {filename}: {e}"))
}

mod claude_code_config {
    use super::*;

    #[test]
    fn mcp_json_uses_project_from_cwd() {
        assert!(read_mcp_template("mcp.json").contains("--project-from-cwd"));
    }

    #[test]
    fn mcp_json_has_no_static_project_path() {
        assert!(!read_mcp_template("mcp.json").contains("${PWD}"));
    }

    #[test]
    fn mcp_json_has_claude_code_context() {
        assert!(read_mcp_template("mcp.json").contains("claude-code"));
    }

    #[test]
    fn mcp_json_starts_serena_by_command() {
        let content = read_mcp_template("mcp.json");
        let v: serde_json::Value =
            serde_json::from_str(&content).expect("mcp.json must be valid JSON");

        assert_eq!(v["serena"]["command"], "serena");
        assert_eq!(
            v["serena"]["args"],
            serde_json::json!([
                "start-mcp-server",
                "--context",
                "claude-code",
                "--project-from-cwd"
            ])
        );

        assert!(!content.contains("uvx"), "mcp.json must not reference uvx");
        assert!(
            !content.contains("git+https://github.com/oraios/serena"),
            "mcp.json must not fetch serena from git"
        );
    }

    #[test]
    fn mcp_json_uses_flat_format() {
        let content = read_mcp_template("mcp.json");
        let v: serde_json::Value =
            serde_json::from_str(&content).expect("mcp.json must be valid JSON");
        assert!(
            v.get("mcpServers").is_none(),
            "mcp.json must not have a top-level mcpServers wrapper — use flat format"
        );
        assert!(
            v.get("serena").is_some(),
            "mcp.json must declare serena server"
        );
        assert!(
            v.get("context7").is_some(),
            "mcp.json must declare context7 server"
        );
    }

    #[test]
    fn built_mcp_json_uses_project_from_cwd() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let build_sh = Path::new(manifest_dir).join("scripts/plugin/build.sh");

        let status = Command::new("bash")
            .arg(build_sh)
            .current_dir(manifest_dir)
            .status()
            .expect("failed to run build.sh");

        assert!(status.success(), "build.sh exited with failure: {status}");

        let output_path =
            Path::new(manifest_dir).join("dist/marketplace/plugins/speq-skill/.mcp.json");
        let content = fs::read_to_string(&output_path)
            .expect("failed to read dist/marketplace/plugins/speq-skill/.mcp.json");

        assert!(
            content.contains("--project-from-cwd"),
            "built .mcp.json should contain --project-from-cwd"
        );
        assert!(
            !content.contains("${PWD}"),
            "built .mcp.json should not contain ${{PWD}}"
        );
        assert!(
            content.contains("claude-code"),
            "built .mcp.json should contain claude-code"
        );

        let v: serde_json::Value =
            serde_json::from_str(&content).expect("built .mcp.json must be valid JSON");
        assert!(
            v.get("mcpServers").is_none(),
            "built .mcp.json must not have a top-level mcpServers wrapper"
        );

        assert_eq!(v["serena"]["command"], "serena");
        assert_eq!(
            v["serena"]["args"],
            serde_json::json!([
                "start-mcp-server",
                "--context",
                "claude-code",
                "--project-from-cwd"
            ])
        );

        let codex_output_path =
            Path::new(manifest_dir).join("dist/marketplace/codex/plugins/speq-skill/.mcp.json");
        let codex_content = fs::read_to_string(&codex_output_path)
            .expect("failed to read dist/marketplace/codex/plugins/speq-skill/.mcp.json");
        let codex_v: serde_json::Value =
            serde_json::from_str(&codex_content).expect("built codex .mcp.json must be valid JSON");
        assert_eq!(codex_v["serena"]["command"], "serena");
        assert_eq!(
            codex_v["serena"]["args"],
            serde_json::json!(["start-mcp-server", "--project-from-cwd", "--context=codex"])
        );

        let plugin_json_path = Path::new(manifest_dir)
            .join("dist/marketplace/plugins/speq-skill/.claude-plugin/plugin.json");
        let plugin_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&plugin_json_path).expect("failed to read built plugin.json"),
        )
        .expect("built plugin.json must be valid JSON");

        let mut expected_agents: Vec<String> =
            fs::read_dir(Path::new(manifest_dir).join(".claude/agents"))
                .expect("failed to read .claude/agents")
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|name| name.ends_with(".md"))
                .collect();
        expected_agents.sort();

        let mut listed_agents: Vec<String> = plugin_json["agents"]
            .as_array()
            .expect("plugin.json agents must be an array")
            .iter()
            .map(|v| v.as_str().unwrap().rsplit('/').next().unwrap().to_string())
            .collect();
        listed_agents.sort();

        assert_eq!(
            listed_agents, expected_agents,
            "plugin.json agents array must list exactly the files in .claude/agents/ \
             (this is the git-agent regression: it shipped in agents/ but wasn't \
             registered in the manifest)"
        );
    }
}

mod codex_config {
    use super::*;

    #[test]
    fn mcp_codex_json_uses_project_from_cwd() {
        assert!(read_mcp_template("mcp-codex.json").contains("--project-from-cwd"));
    }

    #[test]
    fn mcp_codex_json_has_no_static_project_path() {
        assert!(!read_mcp_template("mcp-codex.json").contains("${PWD}"));
    }

    #[test]
    fn mcp_codex_json_has_codex_context() {
        assert!(read_mcp_template("mcp-codex.json").contains("codex"));
    }

    #[test]
    fn mcp_codex_json_starts_serena_by_command() {
        let content = read_mcp_template("mcp-codex.json");
        let v: serde_json::Value =
            serde_json::from_str(&content).expect("mcp-codex.json must be valid JSON");

        assert_eq!(v["serena"]["command"], "serena");
        assert_eq!(
            v["serena"]["args"],
            serde_json::json!(["start-mcp-server", "--project-from-cwd", "--context=codex"])
        );

        assert!(
            !content.contains("uvx"),
            "mcp-codex.json must not reference uvx"
        );
        assert!(
            !content.contains("git+https://github.com/oraios/serena"),
            "mcp-codex.json must not fetch serena from git"
        );
    }

    #[test]
    fn mcp_codex_json_uses_flat_format() {
        let content = read_mcp_template("mcp-codex.json");
        let v: serde_json::Value =
            serde_json::from_str(&content).expect("mcp-codex.json must be valid JSON");
        assert!(
            v.get("mcpServers").is_none(),
            "mcp-codex.json must not have a top-level mcpServers wrapper — use flat format"
        );
        assert!(
            v.get("serena").is_some(),
            "mcp-codex.json must declare serena server"
        );
        assert!(
            v.get("context7").is_some(),
            "mcp-codex.json must declare context7 server"
        );
    }
}

// `install.sh` and `scripts/local-install.sh` legitimately still reference the
// literal string `uvx` once each: to *detect* a stale, git-sourced Codex
// registration (`command: uvx`) left over from before this change, so it can
// be replaced with the by-command form. What must never reappear in either
// script is the old invocation itself (`-- uvx --from ...`) or the git URL —
// those would mean the installer still launches Serena that way.
const LEGACY_INVOCATION: &str = "-- uvx --from";
const LEGACY_GIT_SOURCE: &str = "git+https://github.com/oraios/serena";

#[test]
fn installer_files_have_no_git_sourced_serena() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let scripts_detecting_legacy_uvx = ["install.sh", "scripts/local-install.sh"];
    let files_banning_uvx_outright = [
        "scripts/plugin/mcp.json",
        "scripts/plugin/mcp-codex.json",
        "docs/installation.md",
        "docs/mcp-servers.md",
        "README.md",
    ];

    for file in scripts_detecting_legacy_uvx {
        let path = Path::new(manifest_dir).join(file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {file}: {e}"));

        assert!(
            !content.contains(LEGACY_INVOCATION),
            "{file} must not invoke serena via the legacy `uvx --from` form"
        );
        assert!(
            !content.contains(LEGACY_GIT_SOURCE),
            "{file} must not fetch serena from git"
        );
    }

    for file in files_banning_uvx_outright {
        let path = Path::new(manifest_dir).join(file);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {file}: {e}"));

        assert!(!content.contains("uvx"), "{file} must not reference uvx");
        assert!(
            !content.contains(LEGACY_GIT_SOURCE),
            "{file} must not fetch serena from git"
        );
    }
}
