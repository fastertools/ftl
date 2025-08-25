#!/bin/bash

# Script to fix and reset the release process
set -e

echo "🔧 Fixing FTL Release Process"
echo "=============================="

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed. Please install it first."
    exit 1
fi

# Check if we're in the right repository
if ! git remote -v | grep -q "fastertools/ftl"; then
    echo "❌ This script must be run from the FTL repository"
    exit 1
fi

echo ""
echo "📋 Step 1: Closing existing separate release PRs..."
echo "---------------------------------------------------"

# Close existing separate PRs with explanation
PR_NUMBERS=(343 342 316)
for pr in "${PR_NUMBERS[@]}"; do
    echo "Closing PR #$pr..."
    gh pr close "$pr" --comment "Closing this separate release PR. The release process has been reconfigured to use unified PRs with proper Rust workspace dependency management. A new unified release PR will be created automatically." 2>/dev/null || echo "  ⚠️  PR #$pr might already be closed or doesn't exist"
done

echo ""
echo "📋 Step 2: Validating Cargo workspace configuration..."
echo "------------------------------------------------------"

# Validate Cargo workspace
cd sdk
if cargo metadata --format-version 1 > /dev/null 2>&1; then
    echo "✅ Cargo workspace configuration is valid"
    
    # Show workspace members
    echo ""
    echo "Workspace members:"
    cargo metadata --format-version 1 --no-deps | jq -r '.workspace_members[]' | sed 's/^/  - /'
else
    echo "❌ Cargo workspace configuration has errors"
    cargo check
    exit 1
fi
cd ..

echo ""
echo "📋 Step 3: Validating release-please configuration..."
echo "-----------------------------------------------------"

# Check if release-please config is valid JSON
if jq empty release-please-config.json 2>/dev/null; then
    echo "✅ release-please-config.json is valid JSON"
    
    # Check for required plugins
    if jq -e '.plugins | map(select(.type == "cargo-workspace")) | length > 0' release-please-config.json > /dev/null; then
        echo "✅ cargo-workspace plugin is configured"
    else
        echo "❌ cargo-workspace plugin is missing"
    fi
    
    if jq -e '.plugins | map(select(.type == "linked-versions")) | length > 0' release-please-config.json > /dev/null; then
        echo "✅ linked-versions plugin is configured"
    else
        echo "❌ linked-versions plugin is missing"
    fi
    
    if jq -e '."separate-pull-requests" == false' release-please-config.json > /dev/null; then
        echo "✅ Unified pull requests are enabled"
    else
        echo "⚠️  Warning: separate-pull-requests is still enabled"
    fi
else
    echo "❌ release-please-config.json is not valid JSON"
    exit 1
fi

echo ""
echo "📋 Step 4: Checking manifest versions..."
echo "----------------------------------------"

# Display current versions
echo "Current package versions:"
jq -r 'to_entries | .[] | "  - \(.key): v\(.value)"' .release-please-manifest.json

# Check if Rust SDK versions are in sync
RUST_SDK_VERSION=$(jq -r '."sdk/rust"' .release-please-manifest.json)
RUST_MACROS_VERSION=$(jq -r '."sdk/rust-macros"' .release-please-manifest.json)

if [ "$RUST_SDK_VERSION" = "$RUST_MACROS_VERSION" ]; then
    echo "✅ Rust SDK and macros versions are in sync: v$RUST_SDK_VERSION"
else
    echo "⚠️  Warning: Version mismatch - SDK: v$RUST_SDK_VERSION, Macros: v$RUST_MACROS_VERSION"
    echo "   The linked-versions plugin will sync these in the next release"
fi

echo ""
echo "📋 Step 5: Next steps..."
echo "-----------------------"
echo ""
echo "1. Commit the configuration changes:"
echo "   git add -A"
echo "   git commit -m 'fix(release): configure cargo-workspace and linked-versions plugins'"
echo ""
echo "2. Push to main branch:"
echo "   git push origin main"
echo ""
echo "3. Release-please will automatically:"
echo "   - Detect the configuration changes"
echo "   - Create a new unified release PR"
echo "   - Include all pending package updates"
echo "   - Properly manage Rust workspace dependencies"
echo ""
echo "4. Monitor the new PR:"
echo "   - Check that all packages are included"
echo "   - Verify dependency versions are correct"
echo "   - Review the changelog entries"
echo ""
echo "✅ Release process fix completed successfully!"
echo ""
echo "📚 For more details, see: docs/RELEASE_FIX_SUMMARY.md"