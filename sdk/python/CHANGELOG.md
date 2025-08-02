# Changelog

All notable changes to the FTL Python SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of the FTL Python SDK
- Core MCP protocol implementation for Spin HTTP handlers
- Helper classes for creating tool responses (`ToolResponse`)
- Content type helpers (`ToolContent`)
- Type guards for content validation
- Automatic camelCase to snake_case conversion for tool names
- Full compatibility with componentize-py for WebAssembly compilation
- Comprehensive test suite with pytest
- Type hints throughout the codebase
- Support for Python 3.10+

### Developer Experience
- Development tools integration (black, ruff, mypy, pytest)
- Tox configuration for multi-version testing
- GitHub Actions CI/CD pipeline
- PyPI publishing automation with trusted publishers

### Security
- Input validation for all tool handlers
- Safe error handling without exposing internals