#!/bin/bash
set -e

# Start Spin app in background
echo "Starting Spin app..."
spin up --build &
SPIN_PID=$!

# Wait for app to start
echo "Waiting for app to start..."
sleep 5

# Run tests
echo "Running integration tests..."

# Test 1: Unauthenticated request should return 401
echo "Test 1: Unauthenticated request"
response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/mcp)
if [ "$response" = "401" ]; then
    echo "✓ Unauthenticated request returned 401"
else
    echo "✗ Expected 401, got $response"
    exit 1
fi

# Test 2: OPTIONS request should return 204
echo "Test 2: CORS preflight request"
response=$(curl -s -o /dev/null -w "%{http_code}" -X OPTIONS http://localhost:3000/mcp)
if [ "$response" = "204" ]; then
    echo "✓ OPTIONS request returned 204"
else
    echo "✗ Expected 204, got $response"
    exit 1
fi

# Test 3: OAuth discovery endpoint
echo "Test 3: OAuth discovery endpoint"
response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/.well-known/oauth-authorization-server)
if [ "$response" = "200" ]; then
    echo "✓ OAuth discovery endpoint returned 200"
else
    echo "✗ Expected 200, got $response"
    exit 1
fi

# Test 4: Invalid token should return 401
echo "Test 4: Invalid token"
response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer invalid.token.here" http://localhost:3000/mcp)
if [ "$response" = "401" ]; then
    echo "✓ Invalid token returned 401"
else
    echo "✗ Expected 401, got $response"
    exit 1
fi

# Clean up
echo "Stopping Spin app..."
kill $SPIN_PID
wait $SPIN_PID 2>/dev/null || true

echo "All tests passed!"