# Releasing the FTL Go SDK

This guide explains how to release a new version of the FTL Go SDK.

## Prerequisites

- You must have write access to the repository
- You must be on the `main` branch with a clean working directory
- All tests must be passing

## Release Process

### 1. Prepare the Release

1. Review changes since the last release:
   ```bash
   git log --oneline sdk/go/v0.1.0..HEAD -- sdk/go/
   ```

2. Update the CHANGELOG.md with the changes for this release

3. Ensure all tests pass locally:
   ```bash
   cd sdk/go
   make quality
   ```

### 2. Create the Release

1. Go to the [Actions tab](https://github.com/fastertools/ftl-cli/actions) in GitHub

2. Click on "Release Go SDK" workflow

3. Click "Run workflow"

4. Enter the version number (e.g., `0.2.0`)
   - Do NOT include the `v` prefix
   - Follow semantic versioning

5. Check "Is this a pre-release?" if applicable

6. Click "Run workflow"

### 3. Monitor the Release

The workflow will:
- Validate the version format
- Run all tests
- Update version in go.mod
- Create a git tag with `sdk/go/v` prefix
- Create a GitHub release
- Verify the module is available

### 4. Post-Release

After the release is complete:

1. Verify the module is available:
   ```bash
   go get github.com/fastertools/ftl-cli/sdk/go@v0.2.0
   ```

2. Update any documentation that references the SDK version

3. Announce the release in appropriate channels

## Versioning Guidelines

- **Patch releases** (0.1.x): Bug fixes, documentation updates
- **Minor releases** (0.x.0): New features, backwards-compatible changes
- **Major releases** (x.0.0): Breaking changes (note: while in v0.x, breaking changes can happen in minor releases)

## Troubleshooting

### Module not available immediately

The Go module proxy can take a few minutes to update. You can force a refresh:
```bash
GOPROXY=https://proxy.golang.org go list -m github.com/fastertools/ftl-cli/sdk/go@v0.2.0
```

### Tag already exists

If a tag already exists, you'll need to:
1. Delete the tag locally and remotely (if the release failed)
2. Fix any issues
3. Re-run the release workflow

### Tests failing in CI

Ensure that:
- All tests pass locally
- TinyGo compatibility is maintained
- No new dependencies break WASI compilation