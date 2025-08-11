#!/bin/bash
# Example: Using FTL CLI to access MCP endpoints

set -e

echo "MCP Automation Example"
echo "====================="
echo

# Check if authenticated
if ! ftl eng auth status | grep -q "Logged in"; then
    echo "Error: Not authenticated. Please run 'ftl login' first."
    exit 1
fi

echo "✅ Authentication verified"
echo

# Get the access token (use M2M if USE_M2M is set)
echo "Getting access token..."
if [ "${USE_M2M:-false}" = "true" ]; then
    echo "Using M2M authentication"
    TOKEN=$(ftl eng auth token --m2m)
else
    TOKEN=$(ftl eng auth token)
fi

if [ -z "$TOKEN" ]; then
    echo "Error: Failed to get token"
    exit 1
fi

echo "✅ Token obtained (length: ${#TOKEN} characters)"
echo

# Example: Call an MCP endpoint
MCP_ENDPOINT="${MCP_ENDPOINT:-https://your-app.ftl.tools/mcp}"

echo "Example curl command for tools/list:"
echo "------------------------------------"
cat <<EOF
curl -X POST $MCP_ENDPOINT \\
  -H "Authorization: Bearer \$TOKEN" \\
  -H "Content-Type: application/json" \\
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 1
  }'
EOF

echo
echo
echo "Example: Actually calling the endpoint (if MCP_ENDPOINT is set):"
echo "----------------------------------------------------------------"
if [ "$MCP_ENDPOINT" != "https://your-app.ftl.tools/mcp" ]; then
    echo "Calling $MCP_ENDPOINT..."
    response=$(curl -s -X POST "$MCP_ENDPOINT" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 1
      }')
    
    if command -v jq &> /dev/null; then
        echo "$response" | jq .
    else
        echo "$response"
    fi
else
    echo "Set MCP_ENDPOINT environment variable to test actual calls"
fi

echo
echo "Token ready for use in automation!"
echo
echo "Export as environment variable:"
echo "  export MCP_TOKEN=\"$TOKEN\""
echo
echo "Or use directly in scripts:"
echo "  TOKEN=\$(ftl eng auth token)"