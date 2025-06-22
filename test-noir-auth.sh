#!/bin/bash

# Test script for Noir Authentication Flow
# This tests the complete flow from frontend to server

echo "🧪 Testing Noir Authentication Flow"
echo "=================================="

SERVER_URL="http://localhost:8080"
AUTH_ENDPOINT="/api/authenticate-noir"

echo ""
echo "📡 Testing Server API Endpoint..."

# Test 1: Valid authentication
echo "Test 1: Valid credentials (bob / HyliForEver)"
curl -X POST "${SERVER_URL}${AUTH_ENDPOINT}" \
  -H "Content-Type: application/json" \
  -H "x-user: bob@zkpassport" \
  -H "x-session-key: test-session" \
  -H "x-request-signature: test-signature" \
  -d '{
    "username": "bob",
    "user_field": "12345",
    "password_field": "54321", 
    "proof_type": "noir_circuit"
  }' | jq '.'

echo ""
echo "----------------------------------------"

# Test 2: Invalid username
echo "Test 2: Invalid username"
curl -X POST "${SERVER_URL}${AUTH_ENDPOINT}" \
  -H "Content-Type: application/json" \
  -H "x-user: alice@zkpassport" \
  -H "x-session-key: test-session" \
  -H "x-request-signature: test-signature" \
  -d '{
    "username": "alice",
    "user_field": "98765",
    "password_field": "54321",
    "proof_type": "noir_circuit"
  }' | jq '.'

echo ""
echo "----------------------------------------"

# Test 3: Invalid proof type
echo "Test 3: Invalid proof type"
curl -X POST "${SERVER_URL}${AUTH_ENDPOINT}" \
  -H "Content-Type: application/json" \
  -H "x-user: bob@zkpassport" \
  -H "x-session-key: test-session" \
  -H "x-request-signature: test-signature" \
  -d '{
    "username": "bob",
    "user_field": "12345",
    "password_field": "54321",
    "proof_type": "invalid_type"
  }' | jq '.'

echo ""
echo "✅ Noir Authentication API Tests Complete"
echo ""
echo "🔍 Check server logs for detailed proof generation flow"
echo "📋 Next steps:"
echo "   1. Start frontend: cd front && npm run dev"
echo "   2. Try authentication via UI"
echo "   3. Observe network requests in browser dev tools"
echo "   4. Verify no passwords are transmitted or logged" 