#!/bin/bash

echo "🔍 TMUX DIAGNOSTIC TOOL"
echo "======================="
echo ""

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "❌ tmux is not installed"
    echo "Install with: sudo apt install tmux (Ubuntu/Debian) or brew install tmux (macOS)"
    exit 1
fi

# Get tmux version
TMUX_VERSION=$(tmux -V)
echo "✅ tmux installed: $TMUX_VERSION"

# Check if we're inside a tmux session
if [ -n "$TMUX" ]; then
    echo "✅ Currently inside tmux session"
    
    # Get current prefix key
    PREFIX=$(tmux show-options -g prefix | cut -d' ' -f2)
    echo "🔑 Your prefix key: $PREFIX"
    
    # Show key bindings for common commands
    echo ""
    echo "🎯 Key bindings test:"
    echo "   Prefix + o: $(tmux list-keys | grep 'next-layout' | head -1)"
    echo "   Prefix + arrow: $(tmux list-keys | grep 'select-pane -L' | head -1)"
    
else
    echo "📝 Not currently in tmux (will start test session)"
    
    # Start a test session
    tmux new-session -d -s tmux-test
    PREFIX=$(tmux show-options -g prefix | cut -d' ' -f2)
    echo "🔑 Default prefix key: $PREFIX"
    
    # Kill test session
    tmux kill-session -t tmux-test
fi

echo ""
echo "💡 Quick test:"
echo "1. Run: ./start-dev-panes.sh"
echo "2. Try these keys (one at a time):"
echo "   - Press Ctrl+b, then release, then press 'o'"
echo "   - Press Ctrl+b, then release, then press '?'"
echo ""
echo "🔧 If Ctrl+b doesn't work:"
echo "   - Check: tmux show-options -g prefix"
echo "   - Check: tmux list-keys | head -5"
echo "   - Try: Ctrl+a instead of Ctrl+b"
echo ""
echo "📋 Alternative keys to try:"
echo "   - Alt+b + o"
echo "   - Ctrl+a + o" 
echo "   - Check ~/.tmux.conf for custom settings" 