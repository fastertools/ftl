#!/usr/bin/env bash
set -euo pipefail

# Changelog generation script using conventional commits
# Generates AI-friendly, structured changelogs

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Conventional commit types
declare -A COMMIT_TYPES=(
  ["feat"]="✨ Features"
  ["fix"]="🐛 Bug Fixes"
  ["docs"]="📚 Documentation"
  ["style"]="💄 Styles"
  ["refactor"]="♻️ Code Refactoring"
  ["perf"]="⚡ Performance"
  ["test"]="✅ Tests"
  ["build"]="📦 Build System"
  ["ci"]="👷 CI/CD"
  ["chore"]="🔧 Chores"
  ["revert"]="⏪ Reverts"
)

function usage() {
  echo "Usage: $0 <component> [--since-tag <tag>] [--format <format>]"
  echo ""
  echo "Components:"
  echo "  cli, sdk-rust, sdk-typescript, mcp-authorizer, mcp-gateway"
  echo ""
  echo "Options:"
  echo "  --since-tag <tag>    Generate changelog since this tag (default: last release tag)"
  echo "  --format <format>    Output format: markdown (default), json, github"
  echo ""
  echo "Examples:"
  echo "  $0 cli"
  echo "  $0 sdk-rust --since-tag sdk-rust-v0.2.0"
  echo "  $0 cli --format github"
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

COMPONENT=$1
shift

# Parse options
SINCE_TAG=""
FORMAT="markdown"

while [[ $# -gt 0 ]]; do
  case $1 in
    --since-tag)
      SINCE_TAG="$2"
      shift 2
      ;;
    --format)
      FORMAT="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

# Determine tag prefix
case "$COMPONENT" in
  cli)
    TAG_PREFIX="cli-v"
    COMPONENT_NAME="CLI"
    ;;
  sdk-rust)
    TAG_PREFIX="sdk-rust-v"
    COMPONENT_NAME="Rust SDK"
    ;;
  sdk-typescript)
    TAG_PREFIX="sdk-typescript-v"
    COMPONENT_NAME="TypeScript SDK"
    ;;
  mcp-authorizer|mcp-gateway)
    TAG_PREFIX="component-${COMPONENT}-v"
    COMPONENT_NAME="$COMPONENT"
    ;;
  *)
    echo "Unknown component: $COMPONENT"
    usage
    exit 1
    ;;
esac

# Find the last tag if not specified
if [ -z "$SINCE_TAG" ]; then
  SINCE_TAG=$(git tag -l "${TAG_PREFIX}*" --sort=-v:refname | head -1)
  if [ -z "$SINCE_TAG" ]; then
    echo "No previous release tag found, using first commit"
    SINCE_TAG=$(git rev-list --max-parents=0 HEAD)
  fi
fi

echo -e "${BLUE}Generating changelog for $COMPONENT_NAME since $SINCE_TAG${NC}"

# Collect commits
declare -A COMMITS_BY_TYPE
declare -A BREAKING_CHANGES

while IFS= read -r line; do
  HASH=$(echo "$line" | cut -d' ' -f1)
  MESSAGE=$(echo "$line" | cut -d' ' -f2-)
  
  # Parse conventional commit
  if [[ $MESSAGE =~ ^([a-z]+)(\(([^)]+)\))?!?:\ (.+)$ ]]; then
    TYPE="${BASH_REMATCH[1]}"
    SCOPE="${BASH_REMATCH[3]}"
    DESCRIPTION="${BASH_REMATCH[4]}"
    
    # Check for breaking change
    if [[ $MESSAGE == *"!"* ]] || git show -s --format=%b "$HASH" | grep -q "BREAKING CHANGE:"; then
      BREAKING_MSG=$(git show -s --format=%b "$HASH" | grep -A1 "BREAKING CHANGE:" | tail -1)
      BREAKING_CHANGES["$HASH"]="$DESCRIPTION${BREAKING_MSG:+ - $BREAKING_MSG}"
    fi
    
    # Group by type
    if [[ -n "${COMMIT_TYPES[$TYPE]}" ]]; then
      if [ -n "$SCOPE" ]; then
        COMMITS_BY_TYPE["$TYPE"]+="- **$SCOPE**: $DESCRIPTION ($HASH)"$'\n'
      else
        COMMITS_BY_TYPE["$TYPE"]+="- $DESCRIPTION ($HASH)"$'\n'
      fi
    fi
  else
    # Non-conventional commits go to chore
    COMMITS_BY_TYPE["chore"]+="- $MESSAGE ($HASH)"$'\n'
  fi
done < <(git log "$SINCE_TAG"..HEAD --pretty=format:"%h %s" --reverse)

# Generate output based on format
case "$FORMAT" in
  markdown)
    # Breaking changes
    if [[ ${#BREAKING_CHANGES[@]} -gt 0 ]]; then
      echo "### ⚠️ BREAKING CHANGES"
      echo ""
      for hash in "${!BREAKING_CHANGES[@]}"; do
        echo "- ${BREAKING_CHANGES[$hash]}"
      done
      echo ""
    fi
    
    # Commits by type
    for type in feat fix docs style refactor perf test build ci chore revert; do
      if [[ -n "${COMMITS_BY_TYPE[$type]:-}" ]]; then
        echo "### ${COMMIT_TYPES[$type]}"
        echo ""
        echo -n "${COMMITS_BY_TYPE[$type]}"
        echo ""
      fi
    done
    
    # Contributors
    echo "### 👥 Contributors"
    echo ""
    git log "$SINCE_TAG"..HEAD --pretty=format:"%an" | sort -u | while read -r author; do
      echo "- $author"
    done
    ;;
    
  json)
    # JSON format for programmatic use
    echo "{"
    echo "  \"component\": \"$COMPONENT\","
    echo "  \"since\": \"$SINCE_TAG\","
    echo "  \"breaking_changes\": ["
    first=true
    for hash in "${!BREAKING_CHANGES[@]}"; do
      [[ $first == true ]] && first=false || echo ","
      echo -n "    \"${BREAKING_CHANGES[$hash]}\""
    done
    echo ""
    echo "  ],"
    echo "  \"changes\": {"
    first_type=true
    for type in feat fix docs style refactor perf test build ci chore revert; do
      if [[ -n "${COMMITS_BY_TYPE[$type]:-}" ]]; then
        [[ $first_type == true ]] && first_type=false || echo ","
        echo -n "    \"$type\": ["
        first_commit=true
        while IFS= read -r commit; do
          if [[ -n "$commit" ]]; then
            [[ $first_commit == true ]] && first_commit=false || echo ","
            # Remove leading "- " and escape quotes
            commit_clean="${commit:2}"
            commit_escaped="${commit_clean//\"/\\\"}"
            echo -n "      \"$commit_escaped\""
          fi
        done <<< "${COMMITS_BY_TYPE[$type]}"
        echo -n "    ]"
      fi
    done
    echo ""
    echo "  }"
    echo "}"
    ;;
    
  github)
    # GitHub release format
    if [[ ${#BREAKING_CHANGES[@]} -gt 0 ]]; then
      echo "## ⚠️ Breaking Changes"
      echo ""
      for hash in "${!BREAKING_CHANGES[@]}"; do
        echo "- ${BREAKING_CHANGES[$hash]}"
      done
      echo ""
    fi
    
    echo "## What's Changed"
    echo ""
    
    # Group by importance
    for type in feat fix; do
      if [[ -n "${COMMITS_BY_TYPE[$type]:-}" ]]; then
        echo "**${COMMIT_TYPES[$type]}**"
        echo "${COMMITS_BY_TYPE[$type]}"
      fi
    done
    
    # Other changes
    echo "<details>"
    echo "<summary>Other Changes</summary>"
    echo ""
    for type in docs style refactor perf test build ci chore revert; do
      if [[ -n "${COMMITS_BY_TYPE[$type]:-}" ]]; then
        echo "**${COMMIT_TYPES[$type]}**"
        echo "${COMMITS_BY_TYPE[$type]}"
      fi
    done
    echo "</details>"
    echo ""
    
    echo "**Full Changelog**: https://github.com/fastertools/ftl-cli/compare/$SINCE_TAG...${TAG_PREFIX}NEW_VERSION"
    ;;
    
  *)
    echo "Unknown format: $FORMAT"
    exit 1
    ;;
esac