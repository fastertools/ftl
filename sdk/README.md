# FTL SDKs

Language-specific SDKs for building MCP (Model Context Protocol) tools.

## Available SDKs

### [Rust](./rust)
- **Package**: `ftl-sdk` on [crates.io](https://crates.io/crates/ftl-sdk)
- **Features**: Zero-copy JSON handling, procedural macros, async support

### [TypeScript](./typescript)
- **Package**: `ftl-sdk` on [npm](https://www.npmjs.com/package/ftl-sdk)
- **Features**: Full TypeScript types, tree-shaking support, zero dependencies

### [Python](./python)
- **Package**: `ftl-sdk` on [PyPI](https://pypi.org/project/ftl-sdk)
- **Features**: Zero dependencies (only spin-sdk), Python 3.10+ support, type hints

### [Go](./go)
- **Package**: `github.com/fastertools/ftl/sdk/go`
- **Features**: TinyGo WASI support, zero external dependencies, idiomatic Go API

## Quick Start

Each SDK provides the same core functionality:
- Define tool metadata
- Handle MCP requests
- Return structured responses

See individual SDK directories for language-specific documentation and examples.