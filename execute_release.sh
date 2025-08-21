#!/bin/bash
set -e

echo "🚀 FTL v0.8.0 Release Script"
echo "============================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Step 1: Push current branch${NC}"
echo "Running: git push origin feat/go-cli-refactor"
git push origin feat/go-cli-refactor

echo ""
echo -e "${YELLOW}Step 2: Create PR (if required by your workflow)${NC}"
echo "If you need PR approval, run:"
echo -e "${GREEN}gh pr create --title \"feat: Policy-based authorization with Rego\" --body-file PLATFORM_MIGRATION.md --base main${NC}"
echo ""
read -p "Do you need to create a PR first? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]
then
    gh pr create --title "feat: Policy-based authorization with Rego" \
        --body-file PLATFORM_MIGRATION.md \
        --base main
    echo -e "${YELLOW}PR created! Please get it reviewed and merged, then run this script again.${NC}"
    exit 0
fi

echo ""
echo -e "${YELLOW}Step 3: Merge to main${NC}"
read -p "Ready to merge to main? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]
then
    echo "Checking out main..."
    git checkout main
    git pull origin main
    
    echo "Merging feat/go-cli-refactor..."
    git merge feat/go-cli-refactor
    
    echo "Pushing to main..."
    git push origin main
fi

echo ""
echo -e "${YELLOW}Step 4: Push tag${NC}"
echo "Running: git push origin v0.8.0"
git push origin v0.8.0

echo ""
echo -e "${YELLOW}Step 5: Create GitHub Release${NC}"
echo "Creating release with gh CLI..."
gh release create v0.8.0 \
    --title "v0.8.0: Policy-Based Authorization with Rego" \
    --notes-file CHANGELOG_v0.8.0.md

echo ""
echo -e "${GREEN}✅ Release v0.8.0 complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Verify the module is available:"
echo "   go list -m github.com/fastertools/ftl-cli@v0.8.0"
echo ""
echo "2. Send the migration guide to the platform team:"
echo "   - File: PLATFORM_MIGRATION.md"
echo "   - Key message: No code changes needed if using Process() correctly"
echo ""
echo -e "${GREEN}🎉 Congratulations on the release!${NC}"