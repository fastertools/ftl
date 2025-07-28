# Release Process

This document describes the release process for the FTL project.

## Overview

FTL uses an automated release process that ensures consistency, reduces human error, and provides a smooth experience for both maintainers and users.

## Release Types

### 1. Automated Release PR (Recommended)

The easiest way to create a release is using the automated workflow:

1. Go to [Actions → Prepare Release](https://github.com/fastertools/ftl-cli/actions/workflows/prepare-release.yml)
2. Click "Run workflow"
3. Select:
   - **Component**: Which component to release
   - **Version type**: patch, minor, or major
   - **Custom version** (optional): Override automatic versioning
4. Click "Run workflow"

This will:
- Create a release branch
- Bump versions automatically
- Generate release notes on GitHub
- Create a PR with all changes
- Include release checklist

### 2. Manual Release Process

If you prefer manual control:

```bash
# 1. Use the release script
./scripts/release.sh <component> <version>

# 2. Push changes
git push origin main

# 3. Push tag
git push origin <tag-name>
```

## Component Types

### CLI (`cli`)
- Published to: crates.io
- Tag format: `cli-v{version}`
- Version file: `cli/Cargo.toml`

### Rust SDK (`sdk-rust`)
- Published to: crates.io
- Tag format: `sdk-rust-v{version}`
- Version files: `sdk/rust/Cargo.toml`, `sdk/rust-macros/Cargo.toml`

### TypeScript SDK (`sdk-typescript`)
- Published to: npm
- Tag format: `sdk-typescript-v{version}`
- Version file: `sdk/typescript/package.json`

### Components (`mcp-authorizer`, `mcp-gateway`)
- Published to: ghcr.io
- Tag format: `component-{name}-v{version}`
- Version file: `components/{name}/Cargo.toml`

## Automated Features

### When a Release PR is Merged

1. **Auto-tagging**: When a PR with the "release" label is merged, a tag is automatically created
2. **Release workflow triggers**: The appropriate release workflow runs automatically
3. **Artifacts published**: Binaries, packages, and containers are published
4. **GitHub Release created**: With generated release notes

### Release Notes

GitHub automatically generates release notes from pull requests and commits when creating a release. This provides a more scalable and maintainable approach than in-repo text files.

#### Conventional Commit Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test changes
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Other changes

#### Breaking Changes

Mark breaking changes with `!`:
```
feat!: remove deprecated API
```

Or in the body:
```
feat: update API

BREAKING CHANGE: removed deprecated methods
```

## Release Checklist

Before merging a release PR:

- [ ] All CI checks pass
- [ ] Version numbers are correct
- [ ] Release notes will accurately reflect changes
- [ ] Documentation is updated
- [ ] Breaking changes are clearly marked
- [ ] Dependencies are updated if needed

## Version Guidelines

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

### Pre-1.0 Guidelines

While in 0.x versions:
- Breaking changes increment MINOR
- New features can increment MINOR
- Bug fixes increment PATCH

## Troubleshooting

### Release PR Creation Failed

Check:
1. You have permissions to run workflows
2. The component name is valid
3. No uncommitted changes on main

### Auto-tag Failed

Check:
1. PR title follows format: "Release: {Component} v{version}"
2. PR has the "release" label
3. PR was merged (not closed)

### Publishing Failed

Check:
1. Secrets are configured (CARGO_REGISTRY_TOKEN, NPM_TOKEN)
2. Version doesn't already exist
3. Package metadata is valid

## Security

- Release workflows only run on the main branch
- Tags must be created from main
- All artifacts are checksummed
- Signatures where applicable

## Manual Intervention

If automation fails, you can:

1. Create tags manually:
   ```bash
   git tag -a <tag-name> -m "Release message"
   git push origin <tag-name>
   ```

2. Trigger workflows manually from Actions tab

3. Use the release scripts:
   ```bash
   ./scripts/release.sh <component> <version>
   ```

## Best Practices

1. **Use conventional commits**: Makes release notes clearer
2. **Review GitHub release notes**: Ensure accuracy and completeness
3. **Test before releasing**: Run tests locally for critical releases
4. **Coordinate major releases**: Discuss breaking changes with team
5. **Update documentation**: Keep docs in sync with releases

## For AI Agents

When helping with releases:

1. Always use the automated workflow when possible
2. Ensure conventional commit format for clear release notes
3. Check version consistency across related packages
4. Verify no breaking changes in patch releases
5. Update documentation with new features

The release process is designed to be AI-friendly with clear automation and structured data formats.