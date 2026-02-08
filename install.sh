#!/bin/bash

# vamp installer

set -e

# Parse flags
NO_SETUP=false
for arg in "$@"; do
    case "$arg" in --no-setup) NO_SETUP=true ;; esac
done

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
cat << 'LOGO'
                         
 ██╗   ██╗ █████╗ ███╗   ███╗██████╗ 
 ██║   ██║██╔══██╗████╗ ████║██╔══██╗
 ██║   ██║███████║██╔████╔██║██████╔╝
 ╚██╗ ██╔╝██╔══██║██║╚██╔╝██║██╔═══╝ 
  ╚████╔╝ ██║  ██║██║ ╚═╝ ██║██║     
   ╚═══╝  ╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     
                         
LOGO
echo -e "${NC}"
echo -e "${CYAN}Terminal-native Claude Code environment${NC}"
echo ""

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Darwin*) PKG="brew install"; OS_NAME="macOS" ;;
    Linux*)
        if command -v apt &>/dev/null; then
            PKG="sudo apt install -y"
        elif command -v dnf &>/dev/null; then
            PKG="sudo dnf install -y"
        elif command -v pacman &>/dev/null; then
            PKG="sudo pacman -S --noconfirm"
        else
            PKG=""
            echo -e "${YELLOW}No supported package manager found (apt/dnf/pacman).${NC}"
            echo -e "${YELLOW}You will need to install dependencies manually.${NC}"
            echo ""
        fi
        OS_NAME="Linux"
        ;;
    *) echo -e "${RED}Unsupported OS${NC}"; exit 1 ;;
esac

echo -e "${GREEN}OS:${NC} $OS_NAME"
echo ""

# Install locations
BIN_DIR="$HOME/.local/bin"
LIB_DIR="$HOME/.local/share/vamp"
CONFIG_DIR="$HOME/.config/vamp"

mkdir -p "$BIN_DIR" "$LIB_DIR" "$CONFIG_DIR"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ============================================
# Install dependencies
# ============================================

echo -e "${BLUE}Checking dependencies...${NC}"

check() {
    if command -v "$1" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} $1"
        return 0
    else
        echo -e "  ${YELLOW}○${NC} $1 - $PKG $2"
        return 1
    fi
}

# Required
check tmux tmux || { [ -n "$PKG" ] && $PKG tmux || echo -e "    ${RED}Please install tmux manually${NC}"; }
check jq jq || { [ -n "$PKG" ] && $PKG jq || echo -e "    ${RED}Please install jq manually${NC}"; }

# Recommended
echo ""
echo -e "${BLUE}Recommended tools:${NC}"
check fzf fzf || echo -e "    ${YELLOW}Install: ${PKG:-your package manager} fzf${NC}"

# AI coding agents
echo ""
echo -e "${BLUE}AI coding agents:${NC}"
ai_found=0

check_ai_cli() {
    local name="$1"
    local cmd="$2"
    local fallback="$3"
    local install_hint="$4"

    if command -v "$cmd" &>/dev/null; then
        echo -e "  ${GREEN}✓${NC} $name"
        ai_found=$((ai_found + 1))
        return 0
    elif [ -n "$fallback" ] && [ -e "$fallback" ]; then
        echo -e "  ${GREEN}✓${NC} $name (${fallback})"
        ai_found=$((ai_found + 1))
        return 0
    else
        echo -e "  ${YELLOW}○${NC} $name - $install_hint"
        return 1
    fi
}

check_ai_cli "Claude Code" "claude" "" "npm install -g @anthropic-ai/claude-code"
check_ai_cli "Codex" "codex" "" "https://github.com/openai/codex"
check_ai_cli "Cursor" "cursor-agent" "/Applications/Cursor.app" "https://cursor.com"

if [ "$ai_found" -eq 0 ]; then
    echo ""
    echo -e "  ${RED}⚠ No AI coding agents found.${NC}"
    echo -e "  ${RED}  vamp requires at least one (Claude Code recommended).${NC}"
fi

# Beads
echo ""
echo -e "${BLUE}Installing beads...${NC}"
if check bd beads; then
    true
else
    if [[ "$OS_NAME" == "macOS" ]]; then
        echo "  Installing via brew..."
        brew tap steveyegge/beads 2>/dev/null || true
        brew install beads 2>/dev/null || echo -e "  ${YELLOW}Manual: brew tap steveyegge/beads && brew install beads${NC}"
    else
        echo -e "  ${YELLOW}See: https://github.com/steveyegge/beads${NC}"
    fi
fi

# ============================================
# Install vamp
# ============================================

echo ""

# Detect upgrade
if [ -x "$BIN_DIR/vamp" ]; then
    OLD_VERSION=$("$BIN_DIR/vamp" --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
    NEW_VERSION=$(grep -o 'VAMP_VERSION="[^"]*"' "$SCRIPT_DIR/bin/vamp" | cut -d'"' -f2)
    echo -e "${BLUE}Upgrading vamp ${OLD_VERSION} → ${NEW_VERSION}...${NC}"
else
    echo -e "${BLUE}Installing vamp...${NC}"
fi

# Copy main binary
cp "$SCRIPT_DIR/bin/vamp" "$BIN_DIR/vamp"
chmod +x "$BIN_DIR/vamp"
echo -e "  ${GREEN}✓${NC} $BIN_DIR/vamp"

# Copy utils
cp "$SCRIPT_DIR/lib/vamp-utils.sh" "$LIB_DIR/vamp-utils.sh"
echo -e "  ${GREEN}✓${NC} $LIB_DIR/vamp-utils.sh"

# Build and install sidebar
echo ""
echo -e "${BLUE}Installing sidebar...${NC}"

if [ -d "$SCRIPT_DIR/sidebar" ] && command -v cargo &>/dev/null; then
    echo "  Building from source..."
    if cargo build --release --manifest-path "$SCRIPT_DIR/sidebar/Cargo.toml"; then
        cp "$SCRIPT_DIR/sidebar/target/release/vamp-sidebar" "$BIN_DIR/vamp-sidebar"
        chmod +x "$BIN_DIR/vamp-sidebar"
        # Re-sign on macOS to prevent Apple System Policy from killing the binary
        if [[ "$(uname)" == "Darwin" ]] && command -v codesign &>/dev/null; then
            codesign --sign - --force "$BIN_DIR/vamp-sidebar" 2>/dev/null || true
        fi
        echo -e "  ${GREEN}✓${NC} $BIN_DIR/vamp-sidebar"
    else
        echo -e "  ${RED}✗${NC} Sidebar build failed"
        echo -e "    Try: cd sidebar && cargo build --release"
    fi
elif ! command -v cargo &>/dev/null; then
    echo -e "  ${RED}✗${NC} Rust/cargo not found (required to build the sidebar)"
    echo -e "    Install Rust: ${CYAN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    echo -e "    Then re-run this installer."
else
    echo -e "  ${RED}✗${NC} sidebar/ directory not found"
fi

# Create default config
if [ ! -f "$CONFIG_DIR/config" ]; then
    cat > "$CONFIG_DIR/config" << 'EOF'
# vamp configuration

# Claude command and flags
export VAMP_CLAUDE_CMD="claude"
export VAMP_CLAUDE_FLAGS="--dangerously-skip-permissions"

# Agent type commands (override per-type)
# export VAMP_AGENT_CLAUDE_CMD="claude"
# export VAMP_AGENT_CLAUDE_FLAGS="--dangerously-skip-permissions"
# export VAMP_AGENT_CODEX_CMD="codex"
# export VAMP_AGENT_CODEX_FLAGS=""
# export VAMP_AGENT_CURSOR_CMD="cursor"
# export VAMP_AGENT_CURSOR_FLAGS="--cli"

# Projects directory (for vp command)
export VAMP_PROJECTS_DIR="$HOME/Projects"
EOF
    echo -e "  ${GREEN}✓${NC} $CONFIG_DIR/config"
fi

# ============================================
# Shell setup
# ============================================

echo ""
echo -e "${BLUE}Shell setup...${NC}"

SHELL_NAME=$(basename "$SHELL")
case "$SHELL_NAME" in
    zsh)  RC_FILE="$HOME/.zshrc" ;;
    bash) RC_FILE="$HOME/.bashrc" ;;
    *)    RC_FILE="$HOME/.profile" ;;
esac

# Add to PATH if needed (check both current PATH and rc file content)
if [[ ":$PATH:" != *":$BIN_DIR:"* ]] && ! grep -q '\.local/bin' "$RC_FILE" 2>/dev/null; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$RC_FILE"
    echo -e "  ${GREEN}✓${NC} Added $BIN_DIR to PATH"
else
    echo -e "  ${GREEN}✓${NC} PATH already configured"
fi

# Source utils
if ! grep -q "vamp-utils.sh" "$RC_FILE" 2>/dev/null; then
    echo "" >> "$RC_FILE"
    echo "# vamp - Claude Code environment" >> "$RC_FILE"
    echo "[ -f \"$LIB_DIR/vamp-utils.sh\" ] && source \"$LIB_DIR/vamp-utils.sh\"" >> "$RC_FILE"
    echo -e "  ${GREEN}✓${NC} Added vamp-utils to $RC_FILE"
else
    echo -e "  ${GREEN}✓${NC} vamp-utils already sourced in $RC_FILE"
fi

# ============================================
# Configure Claude Code integration
# ============================================

echo ""
echo -e "${BLUE}Configure Claude Code integration...${NC}"
echo ""
echo -e "vamp setup will configure:"
echo -e "  • Claude Code hooks (session start, pre-compaction)"
echo -e "  • Global CLAUDE.md template with beads workflow"
echo -e "  • Recommended permissions for beads and git commands"
echo ""
if [ "$NO_SETUP" = true ]; then
    echo -e "${YELLOW}Skipping setup (--no-setup)${NC}"
elif [ ! -t 0 ]; then
    echo -e "${YELLOW}Non-interactive install detected, skipping setup.${NC}"
    echo -e "${YELLOW}Run 'vamp setup' manually after install.${NC}"
else
    read -p "Run vamp setup now? [Y/n] " setup_confirm
    if [[ ! "$setup_confirm" =~ ^[Nn]$ ]]; then
        echo ""
        # Source the new PATH so vamp is available
        export PATH="$BIN_DIR:$PATH"
        "$BIN_DIR/vamp" setup
    else
        echo ""
        echo -e "${YELLOW}Skipped. Remember to run 'vamp setup' later for full integration.${NC}"
    fi
fi

# ============================================
# Done
# ============================================

echo ""
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}  Installation complete!${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""
echo -e "Restart your shell or run:"
echo -e "  ${YELLOW}source $RC_FILE${NC}"
echo ""
echo -e "Then start vamping:"
echo -e "  ${CYAN}cd ~/Projects/myapp${NC}"
echo -e "  ${CYAN}vamp${NC}"
echo ""
echo -e "Initialize a project:"
echo -e "  ${CYAN}vamp init${NC}"
echo ""
echo -e "Check setup health:"
echo -e "  ${CYAN}vamp doctor${NC}"
echo ""
echo -e "Get help:"
echo -e "  ${CYAN}vamp help${NC}"
echo ""
