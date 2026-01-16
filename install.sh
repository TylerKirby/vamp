#!/bin/bash

# vamp installer

set -e

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
        command -v apt &>/dev/null && PKG="sudo apt install -y"
        command -v dnf &>/dev/null && PKG="sudo dnf install -y"
        command -v pacman &>/dev/null && PKG="sudo pacman -S"
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
check tmux tmux || $PKG tmux

# Recommended
echo ""
echo -e "${BLUE}Recommended tools:${NC}"
check yazi yazi || echo -e "    ${YELLOW}Install: $PKG yazi${NC}"
check htop htop || $PKG htop 2>/dev/null || true
check lazygit lazygit || echo -e "    ${YELLOW}Install: $PKG lazygit${NC}"
check fzf fzf || echo -e "    ${YELLOW}Install: $PKG fzf${NC}"
check jq jq || $PKG jq 2>/dev/null || true

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
echo -e "${BLUE}Installing vamp...${NC}"

# Copy main binary
cp "$SCRIPT_DIR/bin/vamp" "$BIN_DIR/vamp"
chmod +x "$BIN_DIR/vamp"
echo -e "  ${GREEN}✓${NC} $BIN_DIR/vamp"

# Copy utils
cp "$SCRIPT_DIR/lib/vamp-utils.sh" "$LIB_DIR/vamp-utils.sh"
echo -e "  ${GREEN}✓${NC} $LIB_DIR/vamp-utils.sh"

# Create default config
if [ ! -f "$CONFIG_DIR/config" ]; then
    cat > "$CONFIG_DIR/config" << 'EOF'
# vamp configuration

# File viewer: yazi, lf, ranger, nnn
export VAMP_FILE_VIEWER="yazi"

# System monitor: htop, btop, glances  
export VAMP_MONITOR="htop"

# Claude command
export VAMP_CLAUDE_CMD="claude"

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

# Add to PATH if needed
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$RC_FILE"
    echo -e "  ${GREEN}✓${NC} Added $BIN_DIR to PATH"
fi

# Source utils
if ! grep -q "vamp-utils.sh" "$RC_FILE" 2>/dev/null; then
    echo "" >> "$RC_FILE"
    echo "# vamp - Claude Code environment" >> "$RC_FILE"
    echo "[ -f \"$LIB_DIR/vamp-utils.sh\" ] && source \"$LIB_DIR/vamp-utils.sh\"" >> "$RC_FILE"
    echo -e "  ${GREEN}✓${NC} Added vamp-utils to $RC_FILE"
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
