# GitHub Workflows

This directory contains the CI/CD workflows for the FTL project.

## Workflows

### Core Workflows

- **[ci.yml](./ci.yml)** - Main CI pipeline that runs on every push and PR
  - Linting (rustfmt, clippy)
  - Building and testing the CLI
  - Testing TypeScript and Rust SDKs
  - Security audit

- **[release.yml](./release.yml)** - Release pipeline triggered by version tags
  - Builds binaries for Linux, macOS (x86_64 and aarch64)
  - Creates GitHub releases
  - Publishes to crates.io and npm

### Quality Checks

- **[test-templates.yml](./test-templates.yml)** - Tests the project templates
  - Creates projects with `ftl init`
  - Adds components with `ftl add` for all languages
  - Builds and tests each component type

- **[check-sdk-compatibility.yml](./check-sdk-compatibility.yml)** - Ensures SDK versions match templates
  - Verifies Rust SDK version in Rust template
  - Verifies TypeScript SDK version in TypeScript/JavaScript templates
  - Tests template compilation with SDKs

- **[check-versions.yml](./check-versions.yml)** - Checks version consistency
  - Ensures workspace and CLI versions match
  - Warns about SDK version mismatches in templates

- **[check-docs.yml](./check-docs.yml)** - Documentation checks
  - Verifies README links are valid
  - Ensures CLI help matches documentation
  - Checks template READMEs exist

## Dependabot

The [dependabot.yml](../dependabot.yml) configuration keeps dependencies updated:
- Cargo dependencies (weekly)
- npm dependencies for TypeScript SDK (weekly)
- GitHub Actions (weekly)