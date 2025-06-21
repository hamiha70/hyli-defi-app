#!/bin/bash

# Port Cleanup Script for Hyli Development
# Run this before starting Docker services to avoid conflicts

echo "🔧 Cleaning up port conflicts for Hyli development..."

# Check for PostgreSQL on port 5432
POSTGRES_PID=$(sudo lsof -ti:5432 2>/dev/null)
if [ ! -z "$POSTGRES_PID" ]; then
    echo "  ✅ Killing PostgreSQL process on port 5432 (PID: $POSTGRES_PID)"
    sudo kill -9 $POSTGRES_PID
else
    echo "  ✅ Port 5432 is available"
fi

# Check for Hyli node on port 4321
HYLI_PID=$(sudo lsof -ti:4321 2>/dev/null)
if [ ! -z "$HYLI_PID" ]; then
    echo "  ✅ Killing process on port 4321 (PID: $HYLI_PID)"
    sudo kill -9 $HYLI_PID
else
    echo "  ✅ Port 4321 is available"
fi

# Check for server on port 4002
SERVER_PID=$(sudo lsof -ti:4002 2>/dev/null)
if [ ! -z "$SERVER_PID" ]; then
    echo "  ✅ Killing process on port 4002 (PID: $SERVER_PID)"
    sudo kill -9 $SERVER_PID
else
    echo "  ✅ Port 4002 is available"
fi

# Check for frontend on port 5173
FRONTEND_PID=$(sudo lsof -ti:5173 2>/dev/null)
if [ ! -z "$FRONTEND_PID" ]; then
    echo "  ✅ Killing process on port 5173 (PID: $FRONTEND_PID)"
    sudo kill -9 $FRONTEND_PID
else
    echo "  ✅ Port 5173 is available"
fi

echo "🚀 Port cleanup complete! Ready to start development environment."
echo "" 