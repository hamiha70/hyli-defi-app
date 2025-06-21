#!/bin/bash

# Hyli DeFi AMM Development Environment Setup
# Creates tmux session with all necessary terminals

SESSION_NAME="hyli-dev"
PROJECT_DIR="/home/hamiha70/Projects/ZKHack_Berlin_2025/hyli-defi-app"

# Check if session already exists
if tmux has-session -t $SESSION_NAME 2>/dev/null; then
    echo "Session $SESSION_NAME already exists. Attaching..."
    tmux attach-session -t $SESSION_NAME
    exit 0
fi

echo "🚀 Setting up Hyli DeFi development environment..."

# Create new tmux session (detached)
tmux new-session -d -s $SESSION_NAME -c $PROJECT_DIR

# Window 0: Docker Services
tmux rename-window -t $SESSION_NAME:0 "docker"
tmux send-keys -t $SESSION_NAME:0 "echo '🐳 Docker Services Window'" C-m
tmux send-keys -t $SESSION_NAME:0 "echo 'Commands:'" C-m
tmux send-keys -t $SESSION_NAME:0 "echo '  docker-compose up     # Start all services'" C-m
tmux send-keys -t $SESSION_NAME:0 "echo '  docker-compose down   # Stop services'" C-m
tmux send-keys -t $SESSION_NAME:0 "echo '  docker-compose down --volumes --remove-orphans # Full reset'" C-m
tmux send-keys -t $SESSION_NAME:0 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:0 "echo '▶️  Starting Docker services...'" C-m
tmux send-keys -t $SESSION_NAME:0 "docker-compose up" C-m

# Window 1: Server (RISC0)
tmux new-window -t $SESSION_NAME:1 -n "server" -c $PROJECT_DIR
tmux send-keys -t $SESSION_NAME:1 "echo '⚙️  Server Window (RISC0)'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo 'Commands:'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo '  RISC0_DEV_MODE=1 cargo run -p server        # Start server'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo '  rm -rf data && RISC0_DEV_MODE=1 cargo run -p server # Reset + start'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo '  cargo test -p contract1                     # Run unit tests'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:1 "echo '⏳ Waiting for Docker services to start...'" C-m
tmux send-keys -t $SESSION_NAME:1 "echo '💡 Run: RISC0_DEV_MODE=1 cargo run -p server when ready'" C-m

# Window 2: Frontend
tmux new-window -t $SESSION_NAME:2 -n "frontend" -c $PROJECT_DIR
tmux send-keys -t $SESSION_NAME:2 "echo '🎨 Frontend Window'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo 'Commands:'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo '  cd front && bun run dev    # Start frontend'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo '  bun install               # Install dependencies'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:2 "echo 'URLs:'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo '  Frontend: http://localhost:5173/'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo '  Server:   http://localhost:4002/'" C-m
tmux send-keys -t $SESSION_NAME:2 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:2 "cd front"
tmux send-keys -t $SESSION_NAME:2 "echo '▶️  Starting frontend...'" C-m
tmux send-keys -t $SESSION_NAME:2 "bun run dev" C-m

# Window 3: Git & Commands
tmux new-window -t $SESSION_NAME:3 -n "git" -c $PROJECT_DIR
tmux send-keys -t $SESSION_NAME:3 "echo '📋 Git & Commands Window'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo 'Quick Commands:'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo '  git status'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo '  git add . && git commit -m \"message\"'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo '  curl -X POST http://localhost:4002/api/test-amm ...'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:3 "echo '💡 Testing endpoints:'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo '  curl http://localhost:4002/_health'" C-m
tmux send-keys -t $SESSION_NAME:3 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:3 "git status" C-m

# Window 4: Logs & Monitoring
tmux new-window -t $SESSION_NAME:4 -n "logs" -c $PROJECT_DIR
tmux send-keys -t $SESSION_NAME:4 "echo '📊 Logs & Monitoring Window'" C-m
tmux send-keys -t $SESSION_NAME:4 "echo 'Monitoring commands:'" C-m
tmux send-keys -t $SESSION_NAME:4 "echo '  docker-compose logs -f     # Follow docker logs'" C-m
tmux send-keys -t $SESSION_NAME:4 "echo '  netstat -tlnp | grep :4002  # Check server port'" C-m
tmux send-keys -t $SESSION_NAME:4 "echo '  ps aux | grep cargo         # Check running processes'" C-m
tmux send-keys -t $SESSION_NAME:4 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:4 "echo '🔍 System status:'" C-m
tmux send-keys -t $SESSION_NAME:4 "netstat -tlnp | grep -E ':(4001|4002|4321|5173|8081)' || echo 'No services running yet'" C-m

# Set up key bindings info window
tmux new-window -t $SESSION_NAME:5 -n "help" -c $PROJECT_DIR
tmux send-keys -t $SESSION_NAME:5 "clear" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '🎯 Hyli DeFi Development Environment'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '==================================='" C-m
tmux send-keys -t $SESSION_NAME:5 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '📑 Windows:'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  0: docker    - Docker services (port 4321, 4001, 8081)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  1: server    - RISC0 server (port 4002)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  2: frontend  - Bun dev server (port 5173)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  3: git       - Git & testing commands'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  4: logs      - Monitoring & logs'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  5: help      - This help window'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '⌨️  Tmux Keys:'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Ctrl+b + 0-5      # Switch to window 0-5'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Ctrl+b + c        # Create new window'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Ctrl+b + d        # Detach session'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Ctrl+b + [        # Scroll mode (q to exit)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '🔧 Development Workflow:'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  1. Wait for Docker services (window 0)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  2. Start server: RISC0_DEV_MODE=1 cargo run -p server (window 1)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  3. Frontend auto-starts (window 2)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  4. Test at http://localhost:5173/'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '👤 Default Users:'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Built-in: hyli / hylisecure (always available)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '  Custom:   Register at frontend (reset after chain reset)'" C-m
tmux send-keys -t $SESSION_NAME:5 "echo ''" C-m
tmux send-keys -t $SESSION_NAME:5 "echo '🚀 Ready to develop! Switch to window 1 to start server.'" C-m

# Go to server window initially
tmux select-window -t $SESSION_NAME:1

# Attach to the session
echo "✅ Development environment ready!"
echo "🔗 Attaching to tmux session '$SESSION_NAME'..."
echo ""
echo "Quick start:"
echo "  1. Wait for Docker services to start (window 0)"
echo "  2. Switch to window 1 and run: RISC0_DEV_MODE=1 cargo run -p server"
echo "  3. Frontend is already starting in window 2"
echo "  4. Visit http://localhost:5173/ when ready"
echo ""
echo "Press Ctrl+b + 5 for help window"
echo ""

tmux attach-session -t $SESSION_NAME 