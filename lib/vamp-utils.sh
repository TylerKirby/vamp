#!/bin/bash

# vamp-utils.sh: Shell helpers for the vamp workflow
# Add to your .zshrc or .bashrc:
#   source ~/.local/share/vamp/vamp-utils.sh

# ============================================
# Configuration
# ============================================

export VAMP_PROJECTS_DIR="${VAMP_PROJECTS_DIR:-$HOME/Projects}"

# ============================================
# Quick launchers
# ============================================

# Start vamp in current dir
alias v='vamp'
alias vl='vamp list'
alias va='vamp attach'
alias vk='vamp kill'
alias vi='vamp init'

# Project picker with fzf
vp() {
    if [ -z "$1" ]; then
        if command -v fzf &> /dev/null; then
            local selected=$(find "$VAMP_PROJECTS_DIR" -maxdepth 2 -type d -name ".git" 2>/dev/null | 
                           xargs -I {} dirname {} | 
                           fzf --height 40% --preview 'ls -la {} | head -20')
            [ -n "$selected" ] && vamp "$selected"
        else
            echo "Usage: vp <project-name>"
            echo "Projects in $VAMP_PROJECTS_DIR:"
            ls -1 "$VAMP_PROJECTS_DIR"
        fi
    else
        vamp "$VAMP_PROJECTS_DIR/$1"
    fi
}

# ============================================
# Beads shortcuts
# ============================================

alias bds='bd ready'                    # Show ready tasks
alias bdl='bd list'                     # List all
alias bda='bd list --all'               # List including closed
alias bdb='bd list --status blocked'    # Show blocked
alias bdip='bd list --status in_progress'  # In progress

# Context and sync
alias bdpr='bd prime'                   # Prime context for Claude
alias bdsy='bd sync'                    # Full sync
alias bdco='bd compact --stats'         # Compact with stats

# Create task
bdn() {
    local title="$*"
    [ -z "$title" ] && { echo "Usage: bdn <title>"; return 1; }
    bd create "$title" -t task
}

# Create high-priority task
bdp() {
    local title="$*"
    [ -z "$title" ] && { echo "Usage: bdp <title>"; return 1; }
    bd create "$title" -t task --priority 0
}

# Checkpoint - save progress notes
bdcp() {
    local id="$1"
    shift
    local notes="$*"
    
    if [ -z "$id" ]; then
        echo "Usage: bdcp <issue-id> <notes>"
        echo ""
        echo "Current tasks:"
        bd ready 2>/dev/null || echo "No beads initialized"
        return 1
    fi
    
    if [ -n "$notes" ]; then
        bd update "$id" --notes "$notes"
    fi
    
    bd show "$id"
}

# Quick close
bdd() {
    local id="$1"
    [ -z "$id" ] && { echo "Usage: bdd <issue-id>"; bd ready; return 1; }
    bd close "$id"
}

# ============================================
# Claude Code shortcuts
# ============================================

alias ccr='claude --resume'
alias ccc='claude --continue'
alias ccs='claude --model claude-sonnet-4-20250514'
alias cco='claude --model claude-opus-4-20250514'

# ============================================
# Tmux helpers
# ============================================

# Toggle zoom
alias tz='tmux resize-pane -Z'

# Send command to another pane
pane() {
    local pane="$1"
    shift
    tmux send-keys -t "$pane" "$*" Enter
}

# ============================================
# Workflow helpers
# ============================================

# Morning standup - what's ready?
standup() {
    echo "📿 Ready tasks:"
    bd ready 2>/dev/null || echo "  No beads"
    echo ""
    echo "📝 Recent commits:"
    git log --oneline -5 2>/dev/null || echo "  No git"
    echo ""
    echo "🔄 Git status:"
    git status -s 2>/dev/null || echo "  No git"
}

# End of session - checkpoint everything
eod() {
    echo "💾 End of day checkpoint"
    echo ""

    # Show what's in progress
    echo "In progress tasks:"
    bd list --status in_progress 2>/dev/null || echo "  None"
    echo ""

    read -p "Checkpoint notes (or Enter to skip): " notes
    if [ -n "$notes" ]; then
        # Get first in-progress task and update it
        local task=$(bd list --status in_progress --json 2>/dev/null | jq -r '.[0].id' 2>/dev/null)
        if [ -n "$task" ] && [ "$task" != "null" ]; then
            bd update "$task" --notes "$notes"
            echo "Updated: $task"
        fi
    fi

    echo ""
    echo "Git status:"
    git status -s
}

# Session start - prime context and show status
session_start() {
    echo "🎹 Starting dev session..."
    echo ""

    # Prime beads context if available
    if command -v bd &> /dev/null && [ -d ".beads" -o -f ".beads.jsonl" ]; then
        echo "📿 Priming beads context..."
        bd prime
        echo ""
        echo "📋 Ready tasks:"
        bd ready
    fi

    echo ""
    echo "📝 Recent commits:"
    git log --oneline -5 2>/dev/null || echo "  No git"
    echo ""
    echo "🔄 Git status:"
    git status -s 2>/dev/null || echo "  No git"
}

# Session end - sync and show status
session_end() {
    echo "💾 Ending session..."
    echo ""

    if command -v bd &> /dev/null && [ -d ".beads" -o -f ".beads.jsonl" ]; then
        # Show in-progress
        echo "📋 In progress tasks:"
        bd list --status in_progress 2>/dev/null || echo "  None"
        echo ""

        # Sync beads
        echo "🔗 Syncing beads..."
        bd sync 2>/dev/null || true
    fi

    echo ""
    echo "Git status:"
    git status -s
}

alias ss='session_start'
alias se='session_end'

# ============================================
# Help
# ============================================

vamp-help() {
    cat << 'EOF'
vamp shortcuts
==============

Launcher:
  v              Start vamp (current dir)
  vp [project]   Project picker (fzf)
  va <name>      Attach to session
  vk <name>      Kill session
  vl             List sessions
  vi             Init project

Beads:
  bds            Ready tasks
  bdl            List all
  bdip           In progress
  bdn <title>    New task
  bdp <title>    New P0 task
  bdcp <id> <n>  Checkpoint
  bdd <id>       Close task
  bdpr           Prime context
  bdsy           Sync with git
  bdco           Compact (memory decay)

Claude:
  ccr            Resume session
  ccc            Continue
  ccs            Use Sonnet
  cco            Use Opus

Tmux:
  tz             Toggle zoom
  Ctrl-b z       Toggle zoom
  Ctrl-b d       Detach

Workflow:
  standup        Morning status
  eod            End of day checkpoint
  ss             Session start (prime + status)
  se             Session end (sync + status)
EOF
}

echo "vamp utils loaded. Run 'vamp-help' for shortcuts."
