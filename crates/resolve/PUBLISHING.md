# Publishing ftl-resolve to crates.io

## Prerequisites

1. **Crates.io Account**: Create an account at [crates.io](https://crates.io)
2. **API Token**: Generate an API token from your [crates.io account settings](https://crates.io/settings/tokens)
3. **Cargo Authentication**: Login locally with your token

## Setup Authentication

### Option 1: Using cargo login (Recommended)
```bash
cargo login
# Paste your API token when prompted
```

### Option 2: Set environment variable
```bash
export CARGO_REGISTRY_TOKEN="your-api-token-here"
```

### Option 3: Add to ~/.cargo/credentials.toml
```toml
[registry]
token = "your-api-token-here"
```

## Publishing Process

### 1. Check Current Version
```bash
make version
# or
grep '^version' Cargo.toml
```

### 2. Run Tests and Checks
```bash
make check  # Linting and formatting
make test   # Run all tests
```

### 3. Bump Version (if needed)
```bash
# Bump patch version (0.0.1 -> 0.0.2)
make version BUMP=patch

# Bump minor version (0.0.1 -> 0.1.0)
make version BUMP=minor

# Bump major version (0.0.1 -> 1.0.0)
make version BUMP=major

# Set specific version
make version BUMP=0.2.0
```

### 4. Dry Run (Verify Package)
```bash
make publish-dry
```

This will:
- Verify the package can be built
- Check all metadata is correct
- List all files that will be included
- Confirm the version is not already published

### 5. Publish to crates.io
```bash
make publish
```

This will:
- Run all checks and tests
- Verify the version isn't already published
- Ask for confirmation
- Publish to crates.io
- Provide post-publish instructions

### 6. Create Git Tag
```bash
git tag ftl-resolve-v0.0.1  # Use your version
git push origin ftl-resolve-v0.0.1
```

### 7. Create GitHub Release (Optional)
Use the GitHub workflow:
1. Go to Actions → Release FTL Resolve
2. Click "Run workflow"
3. Enter the version number
4. The workflow will create binaries and a GitHub release

## Troubleshooting

### "Version already exists" Error
The version in Cargo.toml has already been published. Bump the version:
```bash
make version BUMP=patch
```

### Authentication Failed
Ensure you're logged in:
```bash
cargo login
```

### Package Too Large
Check what's being included:
```bash
cargo package --list
```

Add unnecessary files to `.gitignore` or create a `.cargo_vcs_info.json` exclude list.

### Missing Required Fields
Ensure Cargo.toml has all required fields:
- name
- version
- authors or edition
- description
- license or license-file
- repository (recommended)

## Automated Publishing

For automated publishing via GitHub Actions:
1. Add `CRATES_IO_TOKEN` to repository secrets
2. Use the release workflow: `.github/workflows/release-ftl-resolve.yml`
3. Trigger via workflow dispatch or git tag

## Version Management Best Practices

- **Patch** (0.0.x): Bug fixes, documentation updates
- **Minor** (0.x.0): New features, backward-compatible changes
- **Major** (x.0.0): Breaking changes

Follow [Semantic Versioning](https://semver.org/) guidelines.

## Crate Ownership

To add additional owners:
```bash
cargo owner --add github-username ftl-resolve
```

To list current owners:
```bash
cargo owner --list ftl-resolve
```