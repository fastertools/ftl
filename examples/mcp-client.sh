#!/bin/bash
# MCP Client - Examples of calling different MCP methods

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get MCP endpoint from environment or use default
MCP_ENDPOINT="${MCP_ENDPOINT:-https://your-app.ftl.tools/mcp}"

echo -e "${BLUE}MCP Client Examples${NC}"
echo "==================="
echo

# Check authentication
if ! ftl eng auth status | grep -q "Logged in"; then
    echo -e "${RED}Error: Not authenticated. Please run 'ftl login' first.${NC}"
    exit 1
fi

# Get token
echo "Getting authentication token..."
TOKEN=$(ftl eng auth token)
echo -e "${GREEN}✅ Token obtained${NC}"
echo

# Function to call MCP
call_mcp() {
    local method=$1
    local params=${2:-"{}"}
    local id=${3:-1}
    
    echo -e "${BLUE}Calling: $method${NC}"
    
    local response=$(curl -s -X POST "$MCP_ENDPOINT" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"$method\",
            \"params\": $params,
            \"id\": $id
        }")
    
    if command -v jq &> /dev/null; then
        echo "$response" | jq .
    else
        echo "$response"
    fi
    echo
}

# Example 1: List available tools
echo -e "${GREEN}Example 1: List available tools${NC}"
echo "--------------------------------"
call_mcp "tools/list"

# Example 2: Call a specific tool (example)
echo -e "${GREEN}Example 2: Call a specific tool${NC}"
echo "--------------------------------"
# This is an example - adjust the tool name and parameters for your actual tools
call_mcp "tools/call" '{
    "name": "example_tool",
    "arguments": {
        "input": "test"
    }
}' 2

# Example 3: List available prompts
echo -e "${GREEN}Example 3: List prompts${NC}"
echo "------------------------"
call_mcp "prompts/list"

# Example 4: Get a specific prompt
echo -e "${GREEN}Example 4: Get a specific prompt${NC}"
echo "---------------------------------"
call_mcp "prompts/get" '{
    "name": "example_prompt"
}' 4

# Example 5: List resources
echo -e "${GREEN}Example 5: List resources${NC}"
echo "--------------------------"
call_mcp "resources/list"

# Example with error handling
echo -e "${GREEN}Example: With error handling${NC}"
echo "-----------------------------"
response=$(curl -s -w "\n%{http_code}" -X POST "$MCP_ENDPOINT" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {},
        "id": 99
    }')

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" -eq 200 ]; then
    echo -e "${GREEN}Success (HTTP $http_code):${NC}"
    if command -v jq &> /dev/null; then
        echo "$body" | jq .
    else
        echo "$body"
    fi
else
    echo -e "${RED}Error (HTTP $http_code):${NC}"
    echo "$body"
fi

echo
echo -e "${BLUE}Raw curl command for reference:${NC}"
echo "-------------------------------"
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