# Release Scripts

This directory contains automation scripts for managing releases in the FTL monorepo.

## Scripts

### `check-versions.sh`

Shows all component versions and checks for inconsistencies.

```bash
./scripts/check-versions.sh
```

This will display:
- Current versions of all components
- Version dependency mismatches
- Template reference versions
- Recent git tags

### `release.sh`

Automates the release process for individual components.

```bash
# Release the CLI
./scripts/release.sh cli 0.1.0

# Release SDKs
./scripts/release.sh sdk-rust 0.3.0
./scripts/release.sh sdk-typescript 0.3.0
./scripts/release.sh sdk-python 0.3.0      # When available
./scripts/release.sh sdk-go 0.3.0          # When available

# Release components
./scripts/release.sh component mcp-authorizer 0.1.0
./scripts/release.sh component mcp-gateway 0.1.0
```

The script will:
1. Verify you're on the main branch with no uncommitted changes
2. Update version numbers in appropriate Cargo.toml files
3. Create a git commit with the changes
4. Create an appropriately named git tag
5. Provide instructions for pushing the release

## Release Process

### 1. Check Current Versions

```bash
./scripts/check-versions.sh
```

### 2. Run Release Script

```bash
./scripts/release.sh <type> [component] <version>
```

### 3. Push Changes

After reviewing the changes:

```bash
# Push the commits
git push origin main

# Push the tag (this triggers the release workflow)
git push origin <tag-name>
```

### 4. Monitor Release

Check GitHub Actions to ensure the release workflow completes successfully.

## Tag Naming Convention

- CLI: `cli-v{version}` (e.g., `cli-v0.1.0`)
- Rust SDK: `sdk-rust-v{version}` (e.g., `sdk-rust-v0.3.0`)
- TypeScript SDK: `sdk-typescript-v{version}` (e.g., `sdk-typescript-v0.3.0`)
- Python SDK: `sdk-python-v{version}` (e.g., `sdk-python-v0.3.0`)
- Go SDK: `sdk-go-v{version}` (e.g., `sdk-go-v0.3.0`)
- Components: `component-{name}-v{version}` (e.g., `component-mcp-authorizer-v0.1.0`)

## Version Guidelines

- Follow [Semantic Versioning](https://semver.org/)
- Major version (1.0.0): Breaking changes
- Minor version (0.1.0): New features, backwards compatible
- Patch version (0.0.1): Bug fixes, backwards compatible

## Component Independence

Each component can be released independently:
- CLI releases don't affect SDK versions
- Component releases don't affect CLI versions
- SDK releases are independent of each other
- Different language SDKs can have different version numbers

### SDK Publishing Locations

- **Rust SDK**: Published to [crates.io](https://crates.io/crates/ftl-sdk)
- **TypeScript SDK**: Published to [npm](https://www.npmjs.com/package/ftl-sdk)
- **Python SDK**: Will be published to [PyPI](https://pypi.org/project/ftl-sdk/)
- **Go SDK**: Versioned via git tags (no central registry)

## Troubleshooting

### Version Mismatch Warnings

If `check-versions.sh` shows warnings:
1. Update the mismatched versions manually
2. For template references, update after releasing the component
3. For SDK dependencies, ensure versions are synchronized

### Failed Releases

If a release fails:
1. Check GitHub Actions logs
2. Fix any issues
3. Delete the tag if needed: `git tag -d <tag-name>`
4. Delete remote tag: `git push origin :refs/tags/<tag-name>`
5. Try again