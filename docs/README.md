# FTL Documentation

## Overview

This directory contains documentation for the FTL project.

## Documents

- [Version Compatibility Matrix](VERSION_MATRIX.md) - Component version compatibility and upgrade guide
- [Architecture](architecture.md) - System architecture and design decisions
- [Contributing](../CONTRIBUTING.md) - How to contribute to the project
- [Releasing](RELEASING.md) - How to create releases (automated and manual)

## Quick Links

- [Main README](../README.md)
- [Changelog](../CHANGELOG.md)
- [Release Scripts](../scripts/README.md)

## For Developers

- Check versions: `./scripts/check-versions.sh`
- Generate changelog: `./scripts/generate-changelog.sh <component>`
- Create release: Use [Prepare Release workflow](https://github.com/fastertools/ftl-cli/actions/workflows/prepare-release.yml)
- Manual release: See [RELEASING.md](RELEASING.md) and [scripts/README.md](../scripts/README.md)
- CI/CD workflows: See [.github/workflows/](../.github/workflows/)