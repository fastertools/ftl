# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of FTL CLI framework
- Core CLI commands: `new`, `build`, `serve`, `deploy`
- Tool management commands: `test`, `watch`, `validate`, `size`
- Toolkit composition with `build-toolkit`, `serve-toolkit`, `deploy-toolkit`
- Hot reload support in `serve` command
- WebAssembly optimization with configurable flags
- Standalone MCP server implementation (ftl-core)
- Runtime abstraction layer (ftl-runtime)
- Comprehensive project templates with Handlebars
- CI/CD pipeline with GitHub Actions
- Multi-platform binary releases
- Automatic crates.io publishing

### Features
- 🚀 Simple tool creation - Just `lib.rs` and `Cargo.toml`, no `spin.toml` needed
- 🔧 Dynamic toolkit composition - Combine multiple tools into a single deployment
- 📦 WebAssembly optimization - Built-in support for size and performance optimization
- 🛠️ Developer-friendly - Hot reload, local serving, and intuitive CLI
- 🏗️ Runtime abstraction - Spin is an implementation detail, not a requirement

### Technical Details
- Minimum supported Rust version: 1.75
- WebAssembly target: wasm32-wasip1
- Memory allocator: talc (2MB default allocation)
- MCP protocol version: JSON-RPC 2.0

## [0.1.0] - TBD

Initial public release.