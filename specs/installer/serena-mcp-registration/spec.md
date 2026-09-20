# Feature: Serena MCP Registration

Ensures the Serena MCP server is started from a one-time, installed CLI rather than fetched from git on every server start, so Serena starts fast and works offline after installation.

## Background

* Serena is distributed as the `serena-agent` PyPI package, installed once with `uv tool install -p 3.13 serena-agent`, exposing a `serena` executable on the user's PATH
* Both the Claude Code and Codex MCP server templates (`scripts/plugin/mcp.json`, `scripts/plugin/mcp-codex.json`) start Serena with `"command": "serena"` and an `args` array beginning with `start-mcp-server`; neither template references `uvx` or a git source
* The installer (`install.sh`, `scripts/local-install.sh`) installs the Serena CLI once, before provisioning the embedding model, via a dedicated step
* Installing the Serena CLI SHALL NOT re-install it when a `serena` executable is already on PATH
* Installing the Serena CLI is non-fatal: when `uv` is unavailable, or `uv tool install` fails, the installer SHALL warn with manual recovery instructions and continue the rest of the installation
* The Codex CLI registration constant `CODEX_SERENA_ADD` holds the exact by-command registration invocation: `codex mcp add serena -- serena start-mcp-server --project-from-cwd --context=codex`
* Registering Serena with the Codex CLI first inspects the existing registration via `codex mcp get serena`: a registration whose `command:` line reads `uvx` is a stale, git-sourced registration from before this change and is replaced; a registration with any other `command:` value is left alone as already registered; no existing registration falls through to a fresh `codex mcp add`

## Scenarios

### Scenario: Claude Code MCP template starts Serena by command

* *GIVEN* the `scripts/plugin/mcp.json` template
* *WHEN* the plugin is built and loaded by Claude Code
* *THEN* the `serena` server entry SHALL declare `"command": "serena"`
* *AND* the `args` array SHALL be `["start-mcp-server", "--context", "claude-code", "--project-from-cwd"]`
* *AND* the template MUST NOT reference `uvx` or a git source for Serena

### Scenario: Codex MCP template starts Serena by command

* *GIVEN* the `scripts/plugin/mcp-codex.json` template
* *WHEN* the plugin is built and loaded by Codex
* *THEN* the `serena` server entry SHALL declare `"command": "serena"`
* *AND* the `args` array SHALL be `["start-mcp-server", "--project-from-cwd", "--context=codex"]`
* *AND* the template MUST NOT reference `uvx` or a git source for Serena

### Scenario: Installer installs the Serena CLI via uv

* *GIVEN* `uv` is available on PATH
* *AND* `serena` is not already on PATH
* *WHEN* the install script reaches the Serena CLI installation step
* *THEN* the script SHALL run `uv tool install -p 3.13 serena-agent`
* *AND* the script SHALL report that the Serena CLI was installed

### Scenario: Installer skips Serena CLI installation when already present

* *GIVEN* a `serena` executable is already on PATH
* *WHEN* the install script reaches the Serena CLI installation step
* *THEN* the script SHALL NOT invoke `uv`
* *AND* the script SHALL report that the Serena CLI is already installed

### Scenario: Installer warns and continues when uv is absent

* *GIVEN* neither `uv` nor `serena` is on PATH
* *WHEN* the install script reaches the Serena CLI installation step
* *THEN* the script SHALL warn that `uv` was not found
* *AND* the script SHALL print the `uv` install command and the manual `uv tool install -p 3.13 serena-agent` command
* *AND* the script SHALL continue with the rest of the installation

### Scenario: Installer warns and continues when uv tool install fails

* *GIVEN* `uv` is available on PATH, `serena` is not, and `uv tool install -p 3.13 serena-agent` fails
* *WHEN* the install script reaches the Serena CLI installation step
* *THEN* the script SHALL warn that the Serena CLI installation failed
* *AND* the script SHALL print the manual `uv tool install -p 3.13 serena-agent` command
* *AND* the script SHALL continue with the rest of the installation

### Scenario: A stale git-sourced Codex registration is replaced

* *GIVEN* the Codex CLI has an existing `serena` MCP server registration whose `command:` value is `uvx`
* *WHEN* the install script registers Codex MCP servers
* *THEN* the script SHALL remove the existing registration with `codex mcp remove serena`
* *AND* the script SHALL register `serena` again using `$CODEX_SERENA_ADD`
* *AND* the script SHALL report that the registration was replaced

### Scenario: An already-correct Codex registration is left alone

* *GIVEN* the Codex CLI has an existing `serena` MCP server registration whose `command:` value is `serena`
* *WHEN* the install script registers Codex MCP servers
* *THEN* the script SHALL NOT remove or re-add the registration
* *AND* the script SHALL report that `serena` is already registered

### Scenario: Removing a stale Codex registration fails

* *GIVEN* the Codex CLI has an existing `serena` MCP server registration whose `command:` value is `uvx`
* *AND* `codex mcp remove serena` fails
* *WHEN* the install script registers Codex MCP servers
* *THEN* the script SHALL warn that the existing registration could not be removed
* *AND* the script SHALL print the manual `codex mcp remove serena` command and `$CODEX_SERENA_ADD`
* *AND* the script SHALL NOT attempt to register `serena` again
