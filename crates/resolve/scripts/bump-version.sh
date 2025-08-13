#!/usr/bin/env bash
# Script to bump version in Cargo.toml

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_DIR/Cargo.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Get current version
CURRENT_VERSION=$(grep '^version' "$CARGO_TOML" | head -1 | cut -d'"' -f2)

echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Check for pre-release suffix
if [[ "$PATCH" == *"-"* ]]; then
    IFS='-' read -r PATCH_NUM PRERELEASE <<< "$PATCH"
    HAS_PRERELEASE=true
else
    PATCH_NUM="$PATCH"
    HAS_PRERELEASE=false
fi

# Determine bump type
BUMP_TYPE="${1:-patch}"

case "$BUMP_TYPE" in
    major)
        NEW_MAJOR=$((MAJOR + 1))
        NEW_MINOR=0
        NEW_PATCH=0
        NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"
        ;;
    minor)
        NEW_MAJOR=$MAJOR
        NEW_MINOR=$((MINOR + 1))
        NEW_PATCH=0
        NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"
        ;;
    patch)
        NEW_MAJOR=$MAJOR
        NEW_MINOR=$MINOR
        NEW_PATCH=$((PATCH_NUM + 1))
        NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"
        ;;
    *)
        # Check if it's a specific version
        if [[ "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-.*)?$ ]]; then
            NEW_VERSION="$BUMP_TYPE"
        else
            echo -e "${RED}Error: Invalid bump type or version${NC}"
            echo "Usage: $0 [major|minor|patch|<specific-version>]"
            echo "Examples:"
            echo "  $0 patch          # Bump patch version"
            echo "  $0 minor          # Bump minor version"
            echo "  $0 major          # Bump major version"
            echo "  $0 1.2.3          # Set specific version"
            echo "  $0 1.2.3-alpha.1  # Set pre-release version"
            exit 1
        fi
        ;;
esac

echo -e "${YELLOW}Bumping version from $CURRENT_VERSION to $NEW_VERSION${NC}"

# Update Cargo.toml
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS sed syntax
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
else
    # GNU sed syntax
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
fi

# Update Cargo.lock if it exists
if [ -f "$PROJECT_DIR/Cargo.lock" ]; then
    echo "Updating Cargo.lock..."
    (cd "$PROJECT_DIR" && cargo update --package ftl-resolve)
fi

echo -e "${GREEN}✓ Version bumped to $NEW_VERSION${NC}"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am \"chore: bump ftl-resolve version to $NEW_VERSION\""
echo "  3. Test: make test"
echo "  4. Publish dry-run: make publish-dry"
echo "  5. Publish: make publish"
echo "  6. Tag: git tag ftl-resolve-v$NEW_VERSION"
echo "  7. Push: git push origin main --tags"