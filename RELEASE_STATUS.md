# Release Process Status

## ✅ What's Been Fixed

1. **Unified Release PRs**: Changed from `separate-pull-requests: true` to `false`
   - Now creates a single PR (#344) instead of multiple conflicting PRs
   
2. **Linked Versions Plugin**: Added to keep ftl-sdk and ftl-sdk-macros versions in sync
   - Both packages now bump to the same version (0.14.0)
   
3. **Removed Invalid Configurations**:
   - Removed cargo-workspace plugin (requires Cargo.toml at repo root)
   - Fixed extra-files type from invalid "cargo-toml" to "toml"
   
4. **Fixed Cargo Structure**:
   - Removed path dependencies (not allowed for crates.io publishing)
   - Reverted workspace version inheritance (release-please needs explicit versions)

## ⚠️ Current Issue

**Dependency Version Not Updating**: The ftl-sdk package depends on ftl-sdk-macros, but when release-please bumps both to 0.14.0, the dependency reference in ftl-sdk's Cargo.toml still points to 0.13.0.

### Attempted Solutions:
1. ✅ Added linked-versions plugin - keeps package versions in sync
2. ✅ Added extra-files configuration with jsonpath - should update the dependency
3. ⚠️  Extra-files not working as expected for inline TOML tables

## 📊 Current State

- **PR Created**: #344 - "chore: release main"
- **Packages Being Released**:
  - Root: 0.13.0
  - sdk-rust: 0.14.0
  - sdk-rust-macros: 0.14.0
- **Status**: PR created but dependency version mismatch needs manual fix

## 🔧 Manual Workaround

Until the extra-files configuration works correctly, you can:

1. After release-please creates the PR
2. Manually update the dependency version in the PR:
   ```bash
   gh pr checkout 344
   # Edit sdk/rust/Cargo.toml to update ftl-sdk-macros version
   git add sdk/rust/Cargo.toml
   git commit -m "fix: update ftl-sdk-macros dependency version"
   git push
   ```

## 📝 Next Steps for Full Automation

### Option 1: Custom Release-Please Plugin
Create a custom plugin that specifically handles Rust workspace dependencies.

### Option 2: Use Pre-commit Hook
Add a pre-commit hook to the release PR that automatically fixes dependency versions.

### Option 3: Investigate TOML JSONPath
The jsonpath `$.dependencies['ftl-sdk-macros'].version` might need different syntax for inline TOML tables.

### Option 4: Use Generic Updater
Instead of TOML type, try using "generic" type with a glob pattern:
```json
{
  "type": "generic", 
  "path": "sdk/rust/Cargo.toml",
  "glob": "ftl-sdk-macros = { version = \"*\""
}
```

## 🎯 Success Criteria

A fully automated release process requires:
1. ✅ Single unified PR for all packages
2. ✅ Synchronized versions for ftl-sdk and ftl-sdk-macros  
3. ⚠️  Automatic dependency version updates
4. ⚠️  All CI checks passing
5. ⚠️  Ready to merge without manual intervention