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

- **[test-e2e.yml](./test-e2e.yml)** - End-to-end tests for each language
  - Tests full ftl workflow: init, add, build, test
  - Runs for Rust, TypeScript, Python, and Go
  - Ensures component scaffolding works correctly

- **[check-versions.yml](./check-versions.yml)** - Checks version consistency
  - Ensures workspace and CLI versions match

- **[check-docs.yml](./check-docs.yml)** - Documentation checks
  - Verifies README links are valid
  - Ensures CLI help matches documentation

## Dependabot

The [dependabot.yml](../dependabot.yml) configuration keeps dependencies updated:
- Cargo dependencies (weekly)
- npm dependencies for TypeScript SDK (weekly)
- GitHub Actions (weekly)