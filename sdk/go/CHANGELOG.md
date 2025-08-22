# Changelog

All notable changes to the FTL Go SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.11.0 (2025-08-22)

## What's Changed
* chore(main): release  cli v0.11.0 by @bowlofarugula in https://github.com/fastertools/ftl/pull/264
* chore(main): release  sdk-rust v0.11.0 by @bowlofarugula in https://github.com/fastertools/ftl/pull/274
* chore(main): release  v0.11.0 by @bowlofarugula in https://github.com/fastertools/ftl/pull/271
* chore(main): release  sdk-typescript v0.11.0 by @bowlofarugula in https://github.com/fastertools/ftl/pull/277
* chore(main): release  sdk-rust v0.11.0 by @bowlofarugula in https://github.com/fastertools/ftl/pull/278


**Full Changelog**: https://github.com/fastertools/ftl/compare/sdk-go-v0.10.0...sdk-go-v0.11.0

## [Unreleased]

### Added
- Initial release of the FTL Go SDK
- Core MCP protocol implementation for Spin HTTP handlers
- Helper functions for creating tool responses (`Text`, `Textf`, `Error`, `Errorf`, `WithStructured`)
- Content type helpers (`TextContent`, `ImageContent`, `AudioContent`, `ResourceContent`)
- Type guards for content types
- Automatic camelCase to snake_case conversion for tool names
- Comprehensive test suite
- TinyGo compatibility for WebAssembly compilation
- Full support for MCP tool metadata including annotations and output schemas

### Security
- All inputs are properly validated before processing
- Error responses maintain security by not exposing internal details
