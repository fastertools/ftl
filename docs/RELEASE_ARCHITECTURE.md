# Release Architecture

## Overview
This document describes the release architecture for the FTL monorepo, which contains multiple components in different languages.

## Design Principles
1. **Work WITH release-please, not against it** - Accept its limitations
2. **Keep it simple** - Avoid complex workarounds
3. **Explicit over implicit** - Clear, obvious behavior
4. **Automate post-release tasks** - Handle what release-please can't

## Components

### Managed by release-please
- **CLI** (`cmd/ftl`) - Go binary
- **SDKs** (`sdk/*`) - Go, Rust, Python, TypeScript
- **Components** (`components/*`) - WASM components

### Version Synchronization

#### Rust Crates Special Case
The Rust SDK consists of two crates that MUST maintain version parity:
- `ftl-sdk` (main crate)
- `ftl-sdk-macros` (proc-macro crate)

**Solution**: 
- Only `sdk/rust` is configured in release-please
- Both crates use exact version matching (`=0.10.0`)
- Post-release workflow syncs versions if needed

#### Scaffold Versions
The `internal/scaffold/versions.json` file cannot be updated by release-please due to path traversal limitations.

**Solution**:
- Accept version mismatch during release PRs
- Post-release workflow updates scaffold versions
- Validation workflow is lenient for release PRs

## Release Process

### 1. Developer commits to main
- Use conventional commits (`feat:`, `fix:`, etc.)
- Changes trigger release-please

### 2. Release-please creates PRs
- Updates package versions
- Updates changelogs
- Updates dependencies in lock files

### 3. CI validates (with leniency)
- Version consistency check allows mismatches for release PRs
- Other checks run normally

### 4. Merge release PR
- Triggers actual release workflows
- Creates GitHub releases
- Publishes packages

### 5. Post-release sync
- Updates scaffold versions
- Syncs Rust crate versions if needed
- Creates follow-up PR with updates

## Known Limitations

### release-please limitations
1. Cannot update files outside package directories
2. Extra-files paths are relative to package directory
3. Cannot use `../` in paths (security restriction)

### Our accommodations
1. Scaffold versions lag behind until post-release sync
2. Rust crates require manual version alignment
3. Validation is lenient for release PRs

## Troubleshooting

### Version Mismatch Errors
**Problem**: CI fails with version mismatch between ftl-sdk and ftl-sdk-macros

**Solution**: Ensure both crates have the same version in main:
```bash
grep "^version" sdk/rust/Cargo.toml sdk/rust-macros/Cargo.toml
```

### Scaffold Version Out of Sync
**Problem**: `internal/scaffold/versions.json` has old versions

**Solution**: This is expected during releases. Post-release workflow will sync.

### Release PR Won't Generate
**Problem**: No release PR appears after commits

**Causes**:
- Commits don't follow conventional format
- No releasable changes (only `chore:` commits)
- release-please workflow failed

## Maintenance

### Adding a New Component
1. Add package configuration to `release-please-config.json`
2. Add version to `.release-please-manifest.json`
3. Update post-release sync if needed

### Changing Version Strategy
1. Update `release-please-config.json`
2. Ensure validation workflow matches strategy
3. Test with a dry-run

## Summary
This architecture prioritizes simplicity and maintainability over perfection. We accept minor inconveniences (like scaffold versions lagging) in exchange for a system that's easy to understand and maintain.