# FTL Version Compatibility Matrix

This document tracks version compatibility between different FTL components.

## Current Stable Versions

| Component | Version | Release Date | Notes |
|-----------|---------|--------------|-------|
| **CLI** |||
| ftl-cli | 0.0.24 | TBD | Main CLI tool |
| **SDKs** |||
| ftl-sdk (Rust) | 0.2.3 | TBD | Rust SDK for tools |
| ftl-sdk-macros | 0.0.1 | TBD | Proc macros for Rust SDK |
| ftl-sdk (TypeScript) | 0.2.3 | TBD | TypeScript/JavaScript SDK |
| ftl-sdk (Python) | TBD | Future | Python SDK (coming soon) |
| ftl-sdk (Go) | TBD | Future | Go SDK (coming soon) |
| **Components** |||
| mcp-authorizer | 0.0.6 | TBD | Auth gateway component |
| mcp-gateway | 0.0.3 | TBD | MCP protocol gateway |

## Compatibility Matrix

### CLI Compatibility

| ftl-cli Version | Compatible SDK | Compatible Components | Minimum Rust |
|-----------------|----------------|-----------------------|--------------|
| 0.0.24+ | 0.2.x | All current versions | 1.86 |
| 0.0.1-0.0.23 | 0.1.x | Legacy versions | 1.75 |

### SDK Dependencies

| ftl-sdk Version | ftl-sdk-macros Version | spin-sdk Version |
|-----------------|------------------------|------------------|
| 0.2.3 | 0.0.1 | 3.1.0 |
| 0.2.0-0.2.2 | N/A | 3.1.0 |
| 0.1.x | N/A | 3.0.0 |

### Component Requirements

| Component | Spin Runtime | WASM Target | SDK Version |
|-----------|--------------|-------------|-------------|
| mcp-authorizer 0.0.6 | 3.0+ | wasm32-wasip1 | N/A |
| mcp-gateway 0.0.3 | 3.0+ | wasm32-wasip1 | 0.2.1+ |

## Version Policy

### Semantic Versioning

All components follow [Semantic Versioning](https://semver.org/):
- **Major**: Breaking changes
- **Minor**: New features, backwards compatible
- **Patch**: Bug fixes, backwards compatible

### Release Cadence

- **CLI**: Released as needed, typically monthly
- **SDK**: Released with new features or fixes
- **Components**: Released independently as needed

### Backwards Compatibility

- **CLI**: Maintains compatibility with projects created by previous versions
- **SDK**: Minor versions are backwards compatible
- **Components**: Can be upgraded independently

## Upgrade Guide

### Upgrading CLI

```bash
cargo install ftl-cli --force
```

### Upgrading SDKs in Your Project

**Rust SDK:**
```toml
[dependencies]
ftl-sdk = "0.2.3"
```

**TypeScript SDK:**
```json
{
  "dependencies": {
    "ftl-sdk": "^0.2.3"
  }
}
```

**Python SDK (future):**
```bash
pip install ftl-sdk==0.3.0
```

**Go SDK (future):**
```bash
go get github.com/fastertools/ftl-cli/sdk/go@v0.3.0
```

### Upgrading Components

Update your `spin.toml`:
```toml
[component.mcp-authorizer]
source = { registry = "ghcr.io", package = "fastertools:mcp-authorizer", version = "0.0.6" }

[component.mcp-gateway]
source = { registry = "ghcr.io", package = "fastertools:mcp-gateway", version = "0.0.3" }
```

## Breaking Changes

### v0.1.0 → v0.2.0 (SDK)
- Added procedural macros support
- Changed from `ftl_mcp` to `ftl_sdk` package name

### Future Breaking Changes
- Will be documented here before release

## Support Policy

- **Latest version**: Full support
- **Previous minor version**: Security fixes only
- **Older versions**: Best effort / community support

## Version Checking

Use the provided script to check all versions:

```bash
./scripts/check-versions.sh
```

This will show:
- Current versions of all components
- Version mismatches
- Dependency conflicts