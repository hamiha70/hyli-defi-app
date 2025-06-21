#!/bin/bash

# Hyli DeFi AMM Development Environment - 4 Panes Layout
# Creates tmux session with 4 panes filling the screen

SESSION_NAME="hyli-dev"
PROJECT_DIR="/home/hamiha70/Projects/ZKHack_Berlin_2025/hyli-defi-app"

# Check if session already exists
if tmux has-session -t $SESSION_NAME 2>/dev/null; then
    echo "Session $SESSION_NAME already exists. Attaching..."
    tmux attach-session -t $SESSION_NAME
    exit 0
fi

echo "🚀 Setting up Hyli DeFi development environment with 4 panes..."

# Create new tmux session with first pane
tmux new-session -d -s $SESSION_NAME -c $PROJECT_DIR

# Configure tmux appearance and borders (compatible with older versions)
tmux set-option -t $SESSION_NAME pane-border-style fg=colour240
tmux set-option -t $SESSION_NAME pane-active-border-style fg=colour33
tmux set-option -t $SESSION_NAME status-bg colour235
tmux set-option -t $SESSION_NAME status-fg colour255

# Enable pane titles (if supported)
tmux set-option -t $SESSION_NAME pane-border-status top 2>/dev/null || true
tmux set-option -t $SESSION_NAME pane-border-format " #P: #{pane_title} " 2>/dev/null || true

# Split window into 4 panes (2x2 grid)
# Split horizontally (top and bottom)
tmux split-window -h -t $SESSION_NAME

# Split top pane vertically
tmux split-window -v -t $SESSION_NAME:0.0

# Split bottom pane vertically  
tmux split-window -v -t $SESSION_NAME:0.1

# Set pane titles for better identification (if supported)
tmux select-pane -t $SESSION_NAME:0.0 -T "🐳 DOCKER" 2>/dev/null || true
tmux select-pane -t $SESSION_NAME:0.1 -T "⚙️  SERVER" 2>/dev/null || true
tmux select-pane -t $SESSION_NAME:0.2 -T "🎨 FRONTEND" 2>/dev/null || true
tmux select-pane -t $SESSION_NAME:0.3 -T "📋 GIT/CMD" 2>/dev/null || true

# Now we have 4 panes:
# 0: top-left (docker)
# 1: bottom-left (server) 
# 2: top-right (frontend)
# 3: bottom-right (git/commands)

# ==== PANE 0: DOCKER SERVICES (top-left) ====
tmux send-keys -t $SESSION_NAME:0.0 "clear" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '🐳 DOCKER SERVICES'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '=================='" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo 'Commands:'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '  docker-compose up'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '  docker-compose down'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '  docker-compose down --volumes --remove-orphans'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0.0 "echo '▶️  Starting services...'" C-m
tmux send-keys -t $SESSION_NAME:0.0 "docker-compose up" C-m

# ==== PANE 1: SERVER/RISC0 (bottom-left) ====
tmux send-keys -t $SESSION_NAME:0.1 "clear" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '⚙️  RISC0 SERVER'" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '==============='" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo 'Commands:'" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '  RISC0_DEV_MODE=1 cargo run -p server'" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '  rm -rf data && RISC0_DEV_MODE=1 cargo run -p server'" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '  cargo test -p contract1'" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0.1 "echo '⏳ Waiting for Docker... Start server when ready.'" C-m

# ==== PANE 2: FRONTEND (top-right) ====
tmux send-keys -t $SESSION_NAME:0.2 "clear" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo '🎨 FRONTEND'" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo '==========='" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo 'URLs:'" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo '  Frontend: http://localhost:5173/'" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo '  Server:   http://localhost:4002/'" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0.2 "cd front" C-m
tmux send-keys -t $SESSION_NAME:0.2 "echo '▶️  Starting frontend...'" C-m
tmux send-keys -t $SESSION_NAME:0.2 "bun run dev" C-m

# ==== PANE 3: GIT & COMMANDS (bottom-right) ====
tmux send-keys -t $SESSION_NAME:0.3 "clear" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '📋 GIT & COMMANDS'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '================='" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo 'Quick commands:'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '  git status'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '  curl http://localhost:4002/_health'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '  curl -X POST http://localhost:4002/api/test-amm ...'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo '👤 Users: hyli/hylisecure (built-in)'" C-m
tmux send-keys -t $SESSION_NAME:0.3 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0.3 "git status" C-m

# Balance the panes to equal sizes
tmux select-layout -t $SESSION_NAME tiled

# Start with the server pane selected
tmux select-pane -t $SESSION_NAME:0.1

# Display tmux version and prefix info
TMUX_VERSION=$(tmux -V)
TMUX_PREFIX=$(tmux show-options -g prefix | cut -d' ' -f2)

# Attach to the session
echo "✅ Development environment ready with 4 panes!"
echo ""
echo "🎯 Layout (with colored borders):"
echo "┌─────────────┬─────────────┐"
echo "│   DOCKER    │  FRONTEND   │"
echo "│  (port 4321)│ (port 5173) │"
echo "├─────────────┼─────────────┤"
echo "│   SERVER    │ GIT/COMMANDS│"
echo "│  (port 4002)│             │"
echo "└─────────────┴─────────────┘"
echo ""
echo "🚀 Quick start:"
echo "  1. Wait for Docker services (top-left)"
echo "  2. In SERVER pane (bottom-left): RISC0_DEV_MODE=1 cargo run -p server"
echo "  3. Frontend auto-starts (top-right)"
echo "  4. Use bottom-right for git/testing"
echo ""
echo "⌨️  Navigation (IMPORTANT - Check your prefix!):"
echo "  Your tmux version: $TMUX_VERSION"
echo "  Your prefix key: $TMUX_PREFIX (usually Ctrl+b)"
echo ""
echo "  [PREFIX] + o           # Rotate through panes (MAIN COMMAND!)"
echo "  [PREFIX] + arrow keys  # Move between panes"
echo "  [PREFIX] + z           # Zoom/unzoom current pane"
echo "  [PREFIX] + d           # Detach (keeps running)"
echo "  [PREFIX] + c           # Create new window"
echo "  [PREFIX] + n/p         # Next/previous window"
echo "  [PREFIX] + ?           # Show all keybindings"
echo ""
echo "🔧 Troubleshooting:"
echo "  • If Ctrl+b doesn't work, try: tmux list-keys | grep prefix"
echo "  • Check if you have a custom .tmux.conf with different prefix"
echo "  • Default is Ctrl+b, some use Ctrl+a"
echo ""
echo "🎨 Visual Features:"
echo "  • Colored borders (grey inactive, blue active)"
echo "  • Pane titles (if your tmux version supports it)"
echo "  • Clear layout separation"
echo ""

tmux attach-session -t $SESSION_NAME 