# Changelog

All notable changes to the FTL Python SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New decorator-based API with `@ftl.tool` for easier tool creation
- Automatic JSON Schema generation from Python type hints
- Automatic return value conversion to MCP format
- Output schema validation with primitive type wrapping
- Async function support - tools can now be defined as `async def` functions
- The SDK automatically detects async functions and handles them appropriately
- Mixed sync/async tools are supported in the same application
- Full test coverage for async functionality
- Core MCP protocol implementation for Spin HTTP handlers
- Helper classes for creating tool responses (`ToolResponse`)
- Content type helpers (`ToolContent`)
- Type guards for content validation
- Automatic camelCase to snake_case conversion for tool names
- Full compatibility with componentize-py for WebAssembly compilation
- Comprehensive test suite with pytest
- Type hints throughout the codebase
- Support for Python 3.10+
- Development tools integration (black, ruff, mypy, pytest)
- Tox configuration for multi-version testing
- GitHub Actions CI/CD pipeline
- PyPI publishing automation with trusted publishers

### Changed
- Primary API is now decorator-based (old `create_tools` still available for compatibility)
- Handler class is now created with `ftl.create_handler()`

### Security
- Input validation for all tool handlers
- Safe error handling without exposing internals