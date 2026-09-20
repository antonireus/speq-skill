#!/bin/bash
# Install speq-skill CLI and Claude/Codex plugin payloads from local build
# Usage: ./scripts/local-install.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
MARKETPLACE_DIR="${PROJECT_ROOT}/dist/marketplace"
INSTALL_DIR="${HOME}/.speq-skill"
CODEX_MARKETPLACE_NAME="speq-skill-local"
CODEX_MARKETPLACE_ROOT="${INSTALL_DIR}/codex"
CODEX_SKILLS_DIR="${CODEX_HOME:-$HOME/.codex}/skills"
CODEX_SERENA_ADD="codex mcp add serena -- serena start-mcp-server --project-from-cwd --context=codex"

cd "$PROJECT_ROOT"

echo "=== Installing speq-skill ==="

register_codex_plugin() {
    if [ ! -f "$CODEX_MARKETPLACE_ROOT/.agents/plugins/marketplace.json" ]; then
        echo "Codex marketplace payload missing, skipping Codex marketplace registration"
        return
    fi

    if command -v codex &> /dev/null; then
        echo "Registering Codex marketplace..."
        codex plugin marketplace remove "$CODEX_MARKETPLACE_NAME" >/dev/null 2>&1 || true
        if codex plugin marketplace add "$CODEX_MARKETPLACE_ROOT" >/dev/null 2>&1; then
            echo "Codex marketplace: ${CODEX_MARKETPLACE_NAME}"
        else
            echo "Codex marketplace registration failed."
            echo "  Run manually: codex plugin marketplace add ${CODEX_MARKETPLACE_ROOT}"
        fi
    else
        echo "Codex CLI not found. Register the marketplace after installing Codex:"
        echo "  codex plugin marketplace add ${CODEX_MARKETPLACE_ROOT}"
    fi
}

register_codex_mcp_servers() {
    if command -v codex &> /dev/null; then
        echo "Registering Codex MCP servers..."

        local serena_get_output
        if serena_get_output=$(codex mcp get serena 2>&1); then
            if echo "$serena_get_output" | grep -qE '^[[:space:]]*command: uvx$'; then
                if codex mcp remove serena >/dev/null 2>&1; then
                    echo "Replacing git-sourced Codex MCP server registration: serena"
                    if $CODEX_SERENA_ADD >/dev/null 2>&1; then
                        echo "Codex MCP server: serena"
                    else
                        echo "Codex MCP server registration failed: serena"
                        echo "  Run manually: $CODEX_SERENA_ADD"
                    fi
                else
                    echo "Failed to remove existing Codex MCP server registration: serena"
                    echo "  Run manually: codex mcp remove serena"
                    echo "  Then: $CODEX_SERENA_ADD"
                fi
            else
                echo "Codex MCP server already registered: serena"
            fi
        elif $CODEX_SERENA_ADD >/dev/null 2>&1; then
            echo "Codex MCP server: serena"
        else
            echo "Codex MCP server registration failed: serena"
            echo "  Run manually: $CODEX_SERENA_ADD"
        fi

        if codex mcp get context7 >/dev/null 2>&1; then
            echo "Codex MCP server already registered: context7"
        elif codex mcp add context7 -- npx -y @upstash/context7-mcp >/dev/null 2>&1; then
            echo "Codex MCP server: context7"
        else
            echo "Codex MCP server registration failed: context7"
            echo "  Run manually: codex mcp add context7 -- npx -y @upstash/context7-mcp"
        fi
    else
        echo "Codex CLI not found. Register MCP servers after installing Codex:"
        echo "  $CODEX_SERENA_ADD"
        echo "  codex mcp add context7 -- npx -y @upstash/context7-mcp"
    fi
}

install_codex_skills() {
    local source_dir="${INSTALL_DIR}/codex/plugins/speq-skill/skills"

    if [ ! -d "$source_dir" ]; then
        echo "Codex skills payload missing, skipping Codex skill installation"
        return
    fi

    mkdir -p "$CODEX_SKILLS_DIR"

    for skill_dir in "$source_dir"/*; do
        [ -d "$skill_dir" ] || continue

        local skill_name
        local target
        skill_name="$(basename "$skill_dir")"
        target="$CODEX_SKILLS_DIR/speq-$skill_name"

        if [ -L "$target" ]; then
            rm -f "$target"
        elif [ -d "$target" ] && [ -f "$target/.speq-skill-managed" ]; then
            rm -rf "$target"
        elif [ -e "$target" ]; then
            echo "Codex skill already exists and is not managed by speq-skill, skipping: $target"
            continue
        fi

        cp -R "$skill_dir" "$target"
        touch "$target/.speq-skill-managed"
    done

    echo "Codex skills: ${CODEX_SKILLS_DIR}"
}

# Install the Serena CLI as a standalone uv tool, so the MCP server template
# can start it by the bare `serena` command instead of a per-start `uvx` git fetch.
install_serena_tool() {
    if command -v serena &> /dev/null; then
        echo "Serena CLI already installed: $(command -v serena)"
        return 0
    fi

    if command -v uv &> /dev/null; then
        echo "Installing Serena CLI via uv..."
        if uv tool install -p 3.13 serena-agent >/dev/null 2>&1; then
            echo "Installed Serena CLI via uv"
        else
            echo "Serena CLI installation failed."
            echo "  Run manually: uv tool install -p 3.13 serena-agent"
        fi
    else
        echo "uv not found. Install uv, then install the Serena CLI:"
        echo "  curl -LsSf https://astral.sh/uv/install.sh | sh"
        echo "  uv tool install -p 3.13 serena-agent"
    fi

    return 0
}

# 1. Build release binary if not present
if [ ! -f "target/release/speq" ]; then
    echo "Building release binary..."
    cargo build --release
fi

# 2. Build plugin/marketplace
echo "Building plugin..."
./scripts/plugin/build.sh

# 3. Install CLI binary
mkdir -p "$BIN_DIR"
echo "Installing CLI to ${BIN_DIR}/speq..."
cp "target/release/speq" "$BIN_DIR/speq"
chmod +x "$BIN_DIR/speq"

# 3b. Install the Serena CLI as a standalone uv tool
install_serena_tool

# 4. Install Claude plugin via marketplace
if command -v claude &> /dev/null; then
    echo "Adding speq-skill marketplace..."
    claude plugin marketplace add "$MARKETPLACE_DIR"

    echo "Installing speq-skill plugin..."
    claude plugin install speq-skill@speq-skill
else
    echo "Claude CLI not found, skipping Claude plugin registration"
fi

# 5. Install Codex plugin payload and skill copies
rm -rf "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
cp -r "dist/marketplace/." "$INSTALL_DIR/"
register_codex_plugin
register_codex_mcp_servers
install_codex_skills

# 6. Verify installation
echo ""
echo "=== Installation complete ==="

if command -v speq &> /dev/null; then
    echo "CLI: $(which speq)"
else
    echo "CLI: ${BIN_DIR}/speq"
    echo "  Add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo "Claude plugin: ${MARKETPLACE_DIR}"
echo "Codex plugin: ${INSTALL_DIR}/codex/plugins/speq-skill"
echo "Codex marketplace: ${CODEX_MARKETPLACE_ROOT}"
echo ""
echo "To uninstall: ./scripts/uninstall.sh"
