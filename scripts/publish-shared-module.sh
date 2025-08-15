#!/bin/bash
set -e

# Script to publish the shared FTL module
# Usage: ./scripts/publish-shared-module.sh [version]
# Example: ./scripts/publish-shared-module.sh v0.1.0

VERSION=${1:-"v0.1.0"}
MODULE_PATH="go/shared/ftl"
TAG_NAME="${MODULE_PATH}/${VERSION}"

echo "Publishing FTL Shared Module ${VERSION}"
echo "======================================="

# Check if we're in the right directory
if [ ! -f "go/shared/ftl/go.mod" ]; then
    echo "Error: Must run from ftl-cli repository root"
    exit 1
fi

# Run tests first
echo "Running tests..."
cd ${MODULE_PATH}
go mod tidy
go test ./... || { echo "Tests failed"; exit 1; }
cd -

# Check if tag already exists
if git rev-parse "${TAG_NAME}" >/dev/null 2>&1; then
    echo "Error: Tag ${TAG_NAME} already exists"
    echo "To delete: git tag -d ${TAG_NAME}"
    echo "To push deletion: git push origin :refs/tags/${TAG_NAME}"
    exit 1
fi

# Create and push tag
echo "Creating tag ${TAG_NAME}..."
git tag -a "${TAG_NAME}" -m "Release FTL Shared Module ${VERSION}

Shared types and synthesis for FTL CLI and platform backend.

Import: github.com/fastertools/ftl-cli/go/shared/ftl@${VERSION}"

echo "Pushing tag to origin..."
git push origin "${TAG_NAME}"

echo ""
echo "✅ Successfully published!"
echo ""
echo "Backend team can now import with:"
echo "  go get github.com/fastertools/ftl-cli/${MODULE_PATH}@${VERSION}"
echo ""
echo "Or add to go.mod:"
echo "  require github.com/fastertools/ftl-cli/${MODULE_PATH} ${VERSION}"