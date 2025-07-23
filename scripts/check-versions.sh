#!/usr/bin/env bash
set -euo pipefail

# Version checking script for FTL monorepo
# Shows all component versions and their dependencies

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function get_version() {
    local file=$1
    grep '^version' "$file" | head -1 | cut -d'"' -f2
}

function check_component() {
    local name=$1
    local path=$2
    local version=$(get_version "$path/Cargo.toml")
    
    echo -e "${BLUE}$name:${NC} v$version"
}

echo -e "${GREEN}=== FTL Component Versions ===${NC}"
echo ""

# CLI
check_component "CLI (ftl-cli)" "$ROOT_DIR/cli"

# SDKs
echo ""
echo -e "${YELLOW}SDKs:${NC}"
check_component "  Rust SDK (ftl-sdk)" "$ROOT_DIR/sdk/rust"
check_component "  Rust SDK Macros (ftl-sdk-macros)" "$ROOT_DIR/sdk/rust-macros"

# Check SDK dependency version
SDK_MACROS_DEP=$(grep 'ftl-sdk-macros' "$ROOT_DIR/sdk/rust/Cargo.toml" | grep version | cut -d'"' -f2)
ACTUAL_MACROS_VERSION=$(get_version "$ROOT_DIR/sdk/rust-macros/Cargo.toml")
if [[ "$SDK_MACROS_DEP" != "$ACTUAL_MACROS_VERSION" ]]; then
    echo -e "    ${YELLOW}⚠️  Warning: ftl-sdk depends on ftl-sdk-macros $SDK_MACROS_DEP but actual version is $ACTUAL_MACROS_VERSION${NC}"
fi

# TypeScript SDK
if [ -d "$ROOT_DIR/sdk/typescript" ]; then
    TS_VERSION=$(node -p "require('$ROOT_DIR/sdk/typescript/package.json').version" 2>/dev/null || echo "unknown")
    echo -e "  ${BLUE}TypeScript SDK (ftl-sdk):${NC} v$TS_VERSION"
fi

# Future SDKs
if [ -d "$ROOT_DIR/sdk/python" ]; then
    check_component "  Python SDK (ftl-sdk)" "$ROOT_DIR/sdk/python"
fi
if [ -d "$ROOT_DIR/sdk/go" ]; then
    echo -e "  ${BLUE}Go SDK (ftl-sdk):${NC} (versioned by git tags)"
fi

# Components
echo ""
echo -e "${YELLOW}WebAssembly Components:${NC}"
check_component "  MCP Authorizer (mcp-authorizer)" "$ROOT_DIR/components/mcp-authorizer"
check_component "  MCP Gateway (mcp-gateway)" "$ROOT_DIR/components/mcp-gateway"

# Internal crates
echo ""
echo -e "${YELLOW}Internal Crates (not published):${NC}"
check_component "  Commands (ftl-commands)" "$ROOT_DIR/crates/commands"
check_component "  Common (ftl-common)" "$ROOT_DIR/crates/common"
check_component "  Language (ftl-language)" "$ROOT_DIR/crates/language"
check_component "  Runtime (ftl-runtime)" "$ROOT_DIR/crates/runtime"

# Check for version mismatches in templates
echo ""
echo -e "${YELLOW}Template References:${NC}"

# Check mcp-authorizer version in templates
TEMPLATE_AUTH=$(grep 'package = "fastertools:mcp-authorizer"' "$ROOT_DIR/templates/ftl-mcp-server/content/spin.toml" | grep -oP 'version = "\K[^"]+')
ACTUAL_AUTH=$(get_version "$ROOT_DIR/components/mcp-authorizer/Cargo.toml")
echo -e "  Template mcp-authorizer: v$TEMPLATE_AUTH"
if [[ "$TEMPLATE_AUTH" != "$ACTUAL_AUTH" ]]; then
    echo -e "    ${YELLOW}⚠️  Warning: Template references v$TEMPLATE_AUTH but actual is v$ACTUAL_AUTH${NC}"
fi

# Check mcp-gateway version in templates  
TEMPLATE_GW=$(grep 'package = "fastertools:mcp-gateway"' "$ROOT_DIR/templates/ftl-mcp-server/content/spin.toml" | grep -oP 'version = "\K[^"]+')
ACTUAL_GW=$(get_version "$ROOT_DIR/components/mcp-gateway/Cargo.toml")
echo -e "  Template mcp-gateway: v$TEMPLATE_GW"
if [[ "$TEMPLATE_GW" != "$ACTUAL_GW" ]]; then
    echo -e "    ${YELLOW}⚠️  Warning: Template references v$TEMPLATE_GW but actual is v$ACTUAL_GW${NC}"
fi

# Git information
echo ""
echo -e "${YELLOW}Git Information:${NC}"
CURRENT_BRANCH=$(git branch --show-current)
echo "  Current branch: $CURRENT_BRANCH"

# List recent tags
echo "  Recent tags:"
git tag -l --sort=-v:refname | grep -E '^(cli-v|sdk-rust-v|component-)' | head -10 | sed 's/^/    /'

echo ""
echo -e "${GREEN}Done!${NC}"