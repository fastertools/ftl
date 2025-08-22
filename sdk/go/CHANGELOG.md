# Changelog

All notable changes to the FTL Go SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.10.1 (2025-08-22)

**Full Changelog**: https://github.com/fastertools/ftl/compare/sdk-go-v0.10.0...sdk-go-v0.10.1

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
