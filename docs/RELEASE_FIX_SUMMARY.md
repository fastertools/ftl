# Release Process Fix Summary

## Problem Analysis

The release process was failing due to several interconnected issues with the release-please configuration for the Rust SDK packages:

### 1. Missing Cargo Workspace Plugin
- The `cargo-workspace` plugin was not configured, preventing proper handling of Rust workspace dependencies
- This caused ftl-sdk and ftl-sdk-macros versions to get out of sync

### 2. Incorrect Dependency Version Updates
- The extra-files configuration used an incorrect jsonpath format
- The dependency version in ftl-sdk for ftl-sdk-macros wasn't being updated properly during releases

### 3. Separate Pull Requests Creating Conflicts
- With `separate-pull-requests: true`, release-please created individual PRs for each package
- This caused coordination issues for interdependent packages like ftl-sdk and ftl-sdk-macros

### 4. Workspace Version Management
- The Rust workspace wasn't using centralized version management
- Each package defined its own version instead of inheriting from workspace

## Applied Fixes

### 1. Release-Please Configuration Updates
- Added `cargo-workspace` plugin to handle Rust workspace dependencies
- Added `linked-versions` plugin to keep ftl-sdk and ftl-sdk-macros versions synchronized
- Changed `separate-pull-requests` to `false` to create unified release PRs
- Fixed extra-files configuration to use proper cargo-toml updater

### 2. Cargo Workspace Configuration
- Added workspace-level version in `/sdk/Cargo.toml`
- Updated both ftl-sdk and ftl-sdk-macros to inherit version from workspace
- Fixed ftl-sdk-macros dependency to use workspace version

## Next Steps

### Immediate Actions Required

1. **Close existing separate PRs**:
   ```bash
   gh pr close 343 --comment "Closing in favor of unified release PR"
   gh pr close 342 --comment "Closing in favor of unified release PR"
   gh pr close 316 --comment "Closing in favor of unified release PR"
   ```

2. **Trigger new release-please run**:
   - Commit these changes to main
   - Release-please will automatically create a new unified PR on next push to main

3. **Verify the new PR**:
   - Check that all packages are included in a single PR
   - Verify ftl-sdk-macros version is properly updated in ftl-sdk dependencies
   - Ensure changelog entries are correct

### Long-term Improvements

1. **Consider using workspace dependencies for all shared deps**:
   ```toml
   [workspace.dependencies]
   ftl-sdk-macros = { version = "0.13.0", path = "rust-macros" }
   ```

2. **Add validation in CI**:
   - Ensure workspace versions stay synchronized
   - Validate that cargo publish --dry-run works before creating release PR

3. **Document the release process**:
   - Add clear documentation about the polyglot release strategy
   - Include troubleshooting guide for common issues

## Testing the Fix

1. **Local Validation**:
   ```bash
   # Check Cargo.toml files are valid
   cd sdk && cargo check
   
   # Verify workspace structure
   cargo metadata --format-version 1 | jq '.workspace_members'
   ```

2. **Release Simulation**:
   ```bash
   # Run release-please locally (dry-run)
   release-please release-pr \
     --repo-url=fastertools/ftl \
     --token=$GITHUB_TOKEN \
     --dry-run
   ```

## Configuration Reference

The fixed configuration now includes:
- **cargo-workspace plugin**: Manages Rust workspace dependencies automatically
- **linked-versions plugin**: Keeps ftl-sdk and ftl-sdk-macros versions in sync
- **Unified PRs**: All packages released together for better coordination
- **Workspace versions**: Centralized version management for Rust packages