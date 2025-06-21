# Hyli Development Workflow Guide

*Optimal development workflows for the Hyli DeFi AMM project*

---

## 🎯 **Quick Reference**

| **Scenario** | **Action Required** | **Commands** |
|--------------|-------------------|--------------|
| **After reboot** | Start everything | `./start-dev-panes.sh` → wait → start server |
| **Contract changes** | Reset state + server | `rm -rf data && RISC0_DEV_MODE=1 cargo run -p server` |
| **UI/Frontend changes** | Nothing! | Auto-reload via Vite |
| **Server logic changes** | Restart server | `Ctrl+C` → `RISC0_DEV_MODE=1 cargo run -p server` |
| **Docker issues** | Reset Docker | `docker-compose down && docker-compose up` |

---

## 🚀 **Development Scripts Comparison**

### **Option A: `./start-dev-panes.sh` (Recommended)**
```bash
./start-dev-panes.sh
```

**✅ Best for Daily Development:**
- **Layout**: 4 panes in 2x2 grid
- **Navigation**: `Ctrl+b + o` to rotate between panes
- **Efficiency**: All services visible at once
- **Compact**: Perfect for single monitor development

**Pane Layout:**
```
┌─────────────┬─────────────┐
│   DOCKER    │  FRONTEND   │
│  (port 4321)│ (port 5173) │
├─────────────┼─────────────┤
│   SERVER    │ GIT/COMMANDS│
│  (port 4002)│             │
└─────────────┴─────────────┘
```

### **Option B: `./start-dev.sh` (Complex Debugging)**
```bash
./start-dev.sh
```

**✅ Best for Complex Debugging:**
- **Layout**: 6 separate tmux windows
- **Navigation**: `Ctrl+b + 0-5` to switch windows
- **Features**: Dedicated logs window, help window
- **Use Cases**: Multi-log monitoring, complex debugging sessions

**Windows:**
- Window 0: Docker services
- Window 1: Server (RISC0)
- Window 2: Frontend
- Window 3: Git & commands
- Window 4: Logs & monitoring
- Window 5: Help & keybindings

### **⚠️ Cannot Use Both Together**
Both scripts create the same tmux session name. Choose one based on your needs.

---

## 🔄 **Startup Workflows**

### **After Reboot - Complete Startup**

```bash
# 1. Start development environment (with automatic port conflict resolution)
./start-dev-panes.sh
# ✅ Automatically prompts for sudo password once to clean up ports
# ✅ Then starts tmux with all panes without further interruption

# 2. Wait for Docker services to start (watch top-left pane)
# Look for: "hyli" service showing as healthy

# 3. Navigate to SERVER pane (bottom-left, or Ctrl+b + o)
RISC0_DEV_MODE=1 cargo run -p server

# 4. Frontend auto-starts in top-right pane
# 5. Open browser → http://localhost:5173
```

**Estimated Times:**
- Docker startup: ~30-60 seconds
- Server startup: ~15-30 seconds
- Frontend startup: ~5-10 seconds

---

## 🔧 **Change-Based Workflows**

### **Scenario 1: Contract Code Changes**

**When:** You modify files in `contracts/contract1/src/` or `contracts/contract2/src/`

**Why Reset Required:** Contract changes modify state structure, making old state incompatible

#### **Full Reset (Safest)**
```bash
# In SERVER pane:
Ctrl+C                                    # Stop server
rm -rf data                               # Clear ALL state
RISC0_DEV_MODE=1 cargo run -p server     # Restart with fresh state
```

#### **Selective Reset (Faster)**
```bash
# In SERVER pane:
Ctrl+C                                    # Stop server
rm -rf data/state_indexer_contract1.bin  # Clear only contract1 state
# OR: rm -rf data/state_indexer_contract2.bin  # Clear only contract2 state
RISC0_DEV_MODE=1 cargo run -p server     # Restart server
```

### **Scenario 2: UI/Frontend Changes**

**When:** You modify files in `front/src/`

**Action Required:** ✨ **NONE!** ✨

```bash
# Frontend has hot-reload via Vite
# Just save your files and refresh browser
# Server keeps running, Docker keeps running
```

**If Frontend Gets Stuck:**
```bash
# In FRONTEND pane (top-right):
Ctrl+C        # Stop frontend
bun run dev   # Restart frontend
```

### **Scenario 3: Server Logic Changes**

**When:** You modify files in `server/src/`

```bash
# In SERVER pane:
Ctrl+C                                # Stop server
RISC0_DEV_MODE=1 cargo run -p server # Restart server (keep state)
```

### **Scenario 4: Docker Configuration Changes**

**When:** You modify `docker-compose.yml` or need to reset blockchain state

```bash
# In DOCKER pane (top-left):
Ctrl+C                                           # Stop docker
docker-compose down --volumes --remove-orphans  # Full cleanup
docker-compose up                                # Restart fresh
```

---

## ⚡ **Speed Optimization**

### **Development Mode (Already Configured)**
```bash
RISC0_DEV_MODE=1  # ✅ You're already using this!
```
**Benefits:**
- ⚡ 90% faster proof generation
- 🔄 Mock proving for rapid iteration
- ⚠️ Warning messages are normal and expected

### **Incremental Testing Strategy**

```bash
# 1. Test compilation without full server restart
cargo check -p contract1                    # Fast compilation check
cargo test -p contract1                     # Run unit tests

# 2. Test server compilation
cargo check -p server                       # Verify server compiles

# 3. Only then restart full server
RISC0_DEV_MODE=1 cargo run -p server       # Full server restart
```

### **Smart State Management**

```bash
# Keep existing state (faster) for:
# ✅ UI changes
# ✅ Server logic changes  
# ✅ Frontend modifications

# Reset state (required) for:
# ❌ Contract logic changes
# ❌ Contract structure changes
# ❌ State format modifications
```

---

## 🛠 **Automation & Troubleshooting**

### **Quick Health Checks**

```bash
# Check what services are running
netstat -tlnp | grep -E ':(4001|4002|4321|5173|8081)'

# Health check endpoints
curl http://localhost:4002/_health          # Your server
curl http://localhost:4321/v1/info         # Hyli node  
curl http://localhost:5173                 # Frontend

# Check Docker services
docker-compose ps
```

### **Common Port Conflicts**

**PostgreSQL Conflict (Port 5432) - AUTOMATED! ✅**
```bash
# ✅ Now handled automatically by ./start-dev-panes.sh and ./start-dev.sh
# Scripts run ./cleanup-ports.sh before starting tmux (one sudo prompt)
# Cleans up ports: 5432 (PostgreSQL), 4321 (Hyli), 4002 (Server), 5173 (Frontend)

# Manual cleanup (if needed):
./cleanup-ports.sh                         # Clean all development ports
# OR individual cleanup:
sudo lsof -ti:5432 | xargs -r sudo kill    # Kill existing PostgreSQL
docker-compose up                           # Then restart
```

**Other Common Conflicts:**
```bash
# Kill process on specific port
sudo lsof -ti:4002 | xargs -r sudo kill    # Server port
sudo lsof -ti:5173 | xargs -r sudo kill    # Frontend port
sudo lsof -ti:4321 | xargs -r sudo kill    # Hyli node port
```

### **Nuclear Reset (When Everything Is Broken)**

```bash
# Stop all processes
pkill -f "cargo run"
pkill -f "bun run"

# Clean Docker completely
docker-compose down --volumes --remove-orphans
docker system prune -f

# Clean local state  
rm -rf data
rm -rf target  # Optional: if build cache is corrupted

# Restart everything
./start-dev-panes.sh
# Then wait and start server manually
```

---

## 🔍 **Advanced Debugging**

### **Log Monitoring Strategies**

```bash
# Monitor server logs in real-time
RISC0_DEV_MODE=1 cargo run -p server | tee server.log

# Search for specific patterns
grep -E "(ERROR|WARN|Failed)" server.log

# Monitor Docker logs
docker-compose logs -f postgres
docker-compose logs -f hyli
```

### **Transaction Testing Workflow**

```bash
# 1. Basic health check
curl http://localhost:4002/_health

# 2. Test with minimal data
curl -X POST http://localhost:4002/api/test-amm \
  -H "Content-Type: application/json" \  
  -H "x-user: alice@contract1" \
  -d '{"wallet_blobs": [
    {"contract_name": "hydentity", "data": [1,2,3,4]},
    {"contract_name": "wallet", "data": [1,2,3,4]}
  ]}'

# 3. Check response and server logs
# 4. Gradually increase complexity
```

---

## 📋 **Daily Development Checklist**

### **Morning Startup:**
- [ ] `./start-dev-panes.sh`
- [ ] Wait for Docker services (watch for "healthy" status)
- [ ] Start server: `RISC0_DEV_MODE=1 cargo run -p server`
- [ ] Open browser: `http://localhost:5173`
- [ ] Test basic endpoint: `curl http://localhost:4002/_health`

### **Before Making Changes:**
- [ ] Identify change type (contract vs UI vs server)
- [ ] Plan reset strategy based on change type
- [ ] Save/commit current working state

### **After Making Changes:**
- [ ] Follow appropriate workflow for change type
- [ ] Test basic functionality
- [ ] Check server logs for errors
- [ ] Verify frontend still loads

### **End of Day:**
- [ ] Commit changes: `git add . && git commit -m "description"`
- [ ] Optional: Leave tmux session running for next day
- [ ] Optional: `tmux detach` (Ctrl+b + d) to preserve session

---

## 🎯 **Workflow Decision Tree**

```
🤔 What did you change?
│
├── 📝 Contract files (contracts/*/src/*.rs)
│   └── ✅ rm -rf data && RISC0_DEV_MODE=1 cargo run -p server
│
├── 🎨 Frontend files (front/src/*)  
│   └── ✅ Nothing! Auto-reload works
│
├── ⚙️ Server files (server/src/*)
│   └── ✅ Ctrl+C → RISC0_DEV_MODE=1 cargo run -p server
│
├── 🐳 Docker config (docker-compose.yml)
│   └── ✅ docker-compose down && docker-compose up
│
├── 💻 Just rebooted computer
│   └── ✅ ./start-dev-panes.sh → wait → start server
│
└── 🔥 Everything is broken
    └── ✅ Nuclear reset workflow (see above)
```

---

## 🎮 **Tmux Quick Reference**

### **Navigation (start-dev-panes.sh)**
```bash
Ctrl+b + o           # Rotate through 4 panes (MAIN COMMAND!)
Ctrl+b + arrow keys  # Move to specific pane direction
Ctrl+b + z           # Zoom/unzoom current pane
```

### **Session Management**
```bash
Ctrl+b + d           # Detach (keeps running in background)
tmux attach           # Reattach to existing session
tmux kill-session    # Kill session completely
```

### **Window Management (start-dev.sh)**
```bash
Ctrl+b + 0-5         # Switch to window 0-5
Ctrl+b + c           # Create new window
Ctrl+b + ,           # Rename current window
```

---

## 🔗 **Related Documentation**

- **[Development-Debugging-Guide.md](./Development-Debugging-Guide.md)** - Error debugging and troubleshooting
- **[AMM-Contract-Architecture.md](./AMM-Contract-Architecture.md)** - Contract design and testing
- **[Current-Status-Summary.md](./Current-Status-Summary.md)** - Project status and achievements
- **[README.md](../README.md)** - Initial setup and installation

---

*This workflow guide is optimized for the Hyli DeFi AMM project during ZKHack Berlin 2025. Keep this reference handy for efficient development! 🚀* 