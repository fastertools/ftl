# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Support for Spin template options in `ftl init` and `ftl setup templates` commands
  - `--git`: Use a Git repository as the template source
  - `--branch`: Specify Git branch (requires `--git`)
  - `--dir`: Use a local directory as the template source
  - `--tar`: Use a tarball as the template source

### Fixed
- Routes specified in `ftl add` now correctly appear in spin.toml (using --value instead of env vars)
- HTTP routes now automatically end with `/mcp` and use kebab-case
- `ftl build` now shows verbose build output like `ftl up --build` does

### Changed
- Removed `ftl project` subcommand entirely - all commands are now at root level
  - `ftl project init` → `ftl init` (already existed)
  - `ftl project serve` → `ftl up` (already existed)
  - `ftl project deploy` → `ftl deploy` (newly added)
  - Removed `ftl project add` (use `ftl add` instead)
- Restructured component creation workflow
  - `ftl init` now creates a project (http-empty container) instead of a single component
  - Added new `ftl add` command for adding components to projects
  - Removed `--language` option from `ftl init` (moved to `ftl add`)
  - This ensures consistent workflow for single and multi-component projects
- Simplified ftl.toml to only contain component metadata
  - Removed unused `[build]`, `[optimization]`, and `[runtime]` sections
  - Build configuration is handled by Makefiles and language-specific tools
  - Runtime configuration belongs in spin.toml
- Initial release of FTL CLI framework
- Core CLI commands: `new`, `build`, `serve`, `deploy`
- Tool management commands: `test`, `watch`, `validate`, `size`
- Toolkit composition with `build-toolkit`, `serve-toolkit`, `deploy-toolkit`
- Hot reload support in `serve` command
- WebAssembly optimization with configurable flags
- Integration with ftl-mcp SDK for MCP server implementation
- Runtime abstraction layer using Spin WebAssembly runtime
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

## [0.0.1] - TBD

Initial public release.