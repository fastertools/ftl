#!/usr/bin/env bash
set -euo pipefail

# Release automation script for FTL monorepo
# Supports independent releases of CLI, SDKs, and components

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Component types
VALID_TYPES=("cli" "sdk-rust" "sdk-typescript" "sdk-python" "sdk-go" "component")
VALID_COMPONENTS=("mcp-authorizer" "mcp-gateway")
VALID_SDKS=("rust" "typescript" "python" "go")

function print_usage() {
    echo "Usage: $0 <type> [component-name] <version>"
    echo ""
    echo "Types:"
    echo "  cli                    Release the CLI"
    echo "  sdk-rust               Release the Rust SDK"
    echo "  sdk-typescript         Release the TypeScript SDK"
    echo "  sdk-python             Release the Python SDK (when available)"
    echo "  sdk-go                 Release the Go SDK (when available)"
    echo "  component <name>       Release a component"
    echo ""
    echo "Valid component names:"
    echo "  mcp-authorizer"
    echo "  mcp-gateway"
    echo ""
    echo "Examples:"
    echo "  $0 cli 0.1.0"
    echo "  $0 sdk-rust 0.3.0"
    echo "  $0 sdk-typescript 0.3.0"
    echo "  $0 component mcp-authorizer 0.1.0"
}

function check_git_status() {
    if [[ -n $(git status -s) ]]; then
        echo -e "${RED}Error: Working directory has uncommitted changes${NC}"
        git status -s
        exit 1
    fi
    
    # Ensure we're on main branch
    CURRENT_BRANCH=$(git branch --show-current)
    if [[ "$CURRENT_BRANCH" != "main" ]]; then
        echo -e "${RED}Error: Not on main branch (current: $CURRENT_BRANCH)${NC}"
        echo "Please switch to main branch: git checkout main"
        exit 1
    fi
    
    # Pull latest changes
    echo -e "${BLUE}Pulling latest changes...${NC}"
    git pull origin main
}

function update_version() {
    local file=$1
    local version=$2
    
    echo -e "${BLUE}Updating version in $file to $version${NC}"
    
    # Use sed to update version in Cargo.toml
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$version\"/" "$file"
    else
        sed -i "s/^version = \".*\"/version = \"$version\"/" "$file"
    fi
}

function update_changelog() {
    local component=$1
    local version=$2
    local date=$(date +%Y-%m-%d)
    
    echo -e "${BLUE}Updating CHANGELOG.md for $component v$version${NC}"
    
    # This is a placeholder - in a real implementation, you'd update the changelog
    echo -e "${YELLOW}Note: Please manually update CHANGELOG.md before pushing${NC}"
}

function create_tag() {
    local tag=$1
    local message=$2
    
    echo -e "${BLUE}Creating tag: $tag${NC}"
    git tag -a "$tag" -m "$message"
}

function release_cli() {
    local version=$1
    
    echo -e "${GREEN}Releasing CLI v$version${NC}"
    
    # Update version
    update_version "$ROOT_DIR/cli/Cargo.toml" "$version"
    
    # Update changelog
    update_changelog "CLI" "$version"
    
    # Commit changes
    git add -A
    git commit -m "chore(cli): release v$version"
    
    # Create tag
    create_tag "cli-v$version" "Release ftl-cli v$version"
    
    echo -e "${GREEN}✅ CLI release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin cli-v$version"
}

function release_sdk_rust() {
    local version=$1
    
    echo -e "${GREEN}Releasing Rust SDK v$version${NC}"
    
    # Update versions
    update_version "$ROOT_DIR/sdk/rust/Cargo.toml" "$version"
    
    # Check if macros version needs updating
    echo -e "${YELLOW}Do you need to update ftl-sdk-macros version? (y/n)${NC}"
    read -r UPDATE_MACROS
    if [[ "$UPDATE_MACROS" == "y" ]]; then
        echo "Enter new version for ftl-sdk-macros:"
        read -r MACROS_VERSION
        update_version "$ROOT_DIR/sdk/rust-macros/Cargo.toml" "$MACROS_VERSION"
        
        # Update dependency in ftl-sdk
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/ftl-sdk-macros = { version = \".*\"/ftl-sdk-macros = { version = \"$MACROS_VERSION\"/" "$ROOT_DIR/sdk/rust/Cargo.toml"
        else
            sed -i "s/ftl-sdk-macros = { version = \".*\"/ftl-sdk-macros = { version = \"$MACROS_VERSION\"/" "$ROOT_DIR/sdk/rust/Cargo.toml"
        fi
    fi
    
    # Update changelog
    update_changelog "Rust SDK" "$version"
    
    # Commit changes
    git add -A
    git commit -m "chore(sdk-rust): release v$version"
    
    # Create tag
    create_tag "sdk-rust-v$version" "Release ftl-sdk v$version"
    
    echo -e "${GREEN}✅ Rust SDK release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin sdk-rust-v$version"
}

function release_component() {
    local component=$1
    local version=$2
    
    # Validate component name
    if [[ ! " ${VALID_COMPONENTS[@]} " =~ " ${component} " ]]; then
        echo -e "${RED}Error: Invalid component name: $component${NC}"
        echo "Valid components: ${VALID_COMPONENTS[*]}"
        exit 1
    fi
    
    echo -e "${GREEN}Releasing component $component v$version${NC}"
    
    # Update version
    update_version "$ROOT_DIR/components/$component/Cargo.toml" "$version"
    
    # Update changelog
    update_changelog "$component" "$version"
    
    # Commit changes
    git add -A
    git commit -m "chore($component): release v$version"
    
    # Create tag
    create_tag "component-$component-v$version" "Release $component v$version"
    
    echo -e "${GREEN}✅ Component $component release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin component-$component-v$version"
}

function release_sdk_typescript() {
    local version=$1
    
    echo -e "${GREEN}Releasing TypeScript SDK v$version${NC}"
    
    # Update version in package.json
    echo -e "${BLUE}Updating version in sdk/typescript/package.json to $version${NC}"
    cd "$ROOT_DIR/sdk/typescript"
    npm version "$version" --no-git-tag-version
    cd "$ROOT_DIR"
    
    # Update changelog
    update_changelog "TypeScript SDK" "$version"
    
    # Commit changes
    git add -A
    git commit -m "chore(sdk-typescript): release v$version"
    
    # Create tag
    create_tag "sdk-typescript-v$version" "Release TypeScript SDK v$version"
    
    echo -e "${GREEN}✅ TypeScript SDK release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin sdk-typescript-v$version"
}

function release_sdk_python() {
    local version=$1
    
    echo -e "${GREEN}Releasing Python SDK v$version${NC}"
    
    if [ ! -d "$ROOT_DIR/sdk/python" ]; then
        echo -e "${RED}Error: Python SDK not found at sdk/python${NC}"
        echo "Python SDK support will be added soon!"
        exit 1
    fi
    
    # Update version in pyproject.toml
    update_version "$ROOT_DIR/sdk/python/pyproject.toml" "$version"
    
    # Update changelog
    update_changelog "Python SDK" "$version"
    
    # Commit changes
    git add -A
    git commit -m "chore(sdk-python): release v$version"
    
    # Create tag
    create_tag "sdk-python-v$version" "Release Python SDK v$version"
    
    echo -e "${GREEN}✅ Python SDK release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin sdk-python-v$version"
}

function release_sdk_go() {
    local version=$1
    
    echo -e "${GREEN}Releasing Go SDK v$version${NC}"
    
    if [ ! -d "$ROOT_DIR/sdk/go" ]; then
        echo -e "${RED}Error: Go SDK not found at sdk/go${NC}"
        echo "Go SDK support will be added soon!"
        exit 1
    fi
    
    # Update changelog
    update_changelog "Go SDK" "$version"
    
    # Note: Go modules use git tags for versioning, no file to update
    
    # Commit changes
    git add -A
    git commit -m "chore(sdk-go): release v$version"
    
    # Create tag
    create_tag "sdk-go-v$version" "Release Go SDK v$version"
    
    echo -e "${GREEN}✅ Go SDK release prepared!${NC}"
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review and update CHANGELOG.md"
    echo "  2. Push changes: git push origin main"
    echo "  3. Push tag: git push origin sdk-go-v$version"
}

# Main script
if [[ $# -lt 2 ]]; then
    print_usage
    exit 1
fi

TYPE=$1

# Check git status first
check_git_status

case "$TYPE" in
    cli)
        if [[ $# -ne 2 ]]; then
            print_usage
            exit 1
        fi
        release_cli "$2"
        ;;
    sdk-rust)
        if [[ $# -ne 2 ]]; then
            print_usage
            exit 1
        fi
        release_sdk_rust "$2"
        ;;
    sdk-typescript)
        if [[ $# -ne 2 ]]; then
            print_usage
            exit 1
        fi
        release_sdk_typescript "$2"
        ;;
    sdk-python)
        if [[ $# -ne 2 ]]; then
            print_usage
            exit 1
        fi
        release_sdk_python "$2"
        ;;
    sdk-go)
        if [[ $# -ne 2 ]]; then
            print_usage
            exit 1
        fi
        release_sdk_go "$2"
        ;;
    component)
        if [[ $# -ne 3 ]]; then
            print_usage
            exit 1
        fi
        release_component "$2" "$3"
        ;;
    *)
        echo -e "${RED}Error: Invalid type: $TYPE${NC}"
        print_usage
        exit 1
        ;;
esac