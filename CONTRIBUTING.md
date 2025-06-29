# Contributing to FTL CLI

First off, thank you for considering contributing to FTL! It's people like you that make FTL such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues as you might find that you don't need to create one. When you are creating a bug report, please include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples to demonstrate the steps**
- **Describe the behavior you observed and what behavior you expected**
- **Include screenshots if relevant**
- **Include your environment details** (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use a clear and descriptive title**
- **Provide a detailed description of the suggested enhancement**
- **Provide specific examples to demonstrate the enhancement**
- **Describe the current behavior and expected behavior**
- **Explain why this enhancement would be useful**

### Pull Requests

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code follows the existing style
6. Issue that pull request!

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Spin CLI (for testing)
- wasm32-wasip1 target: `rustup target add wasm32-wasip1`

### Building

```bash
# Clone the repository
git clone https://github.com/fastertools/ftl-cli
cd ftl-cli

# Build all components
cargo build --all

# Run tests
cargo test --all

# Build for release
cargo build --release
```

### Project Structure

```
ftl-cli/
├── ftl-cli/        # Main CLI implementation
├── ftl-sdk-rs/    # Core MCP server library
├── ftl-runtime/    # Runtime abstraction layer
└── examples/       # Example tools
```

## Coding Guidelines

### Rust Style

- Follow standard Rust naming conventions
- Use `rustfmt` for formatting: `cargo fmt --all`
- Use `clippy` for linting: `cargo clippy --all-targets --all-features -- -D warnings`
- Write descriptive commit messages
- Add documentation comments for public APIs
- Include examples in documentation where appropriate

### Error Handling

- Use `anyhow::Result` for fallible functions in the CLI
- Use `thiserror` for library errors in ftl-sdk-rs and ftl-runtime
- Provide helpful error messages with context
- Chain errors appropriately using `.context()`

### Testing

- Write unit tests for new functionality
- Add integration tests for CLI commands
- Test edge cases and error conditions
- Ensure tests are deterministic and don't depend on external state

Example test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";
        
        // Act
        let result = process(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Documentation

- Add inline documentation for all public items
- Include examples in documentation
- Update README.md if adding new features
- Add entries to CHANGELOG.md for notable changes

## Architecture Decisions

### Key Design Principles

1. **Simplicity First**: Make the common case easy
2. **WebAssembly Optimized**: Every decision should consider WASM performance
3. **Tool Isolation**: One tool per server for security and performance
4. **Runtime Abstraction**: Spin is an implementation detail, not a core dependency

### Adding New Commands

When adding a new CLI command:

1. Create a new module in `ftl-cli/src/commands/`
2. Add the command to the `Command` enum in `main.rs`
3. Export the module in `commands/mod.rs`
4. Implement the `execute` function
5. Add tests for the command
6. Update the README with the new command

### Modifying Core APIs

Changes to `ftl-sdk-rs` APIs require careful consideration:

1. Maintain backward compatibility when possible
2. Document breaking changes clearly
3. Update all dependent code
4. Add migration guides for breaking changes

## Release Process

1. Update version numbers in all `Cargo.toml` files
2. Update CHANGELOG.md with release notes
3. Create a git tag: `git tag -a v0.x.x -m "Release v0.x.x"`
4. Push the tag: `git push origin v0.x.x`
5. GitHub Actions will automatically create releases and publish to crates.io

## Getting Help

- Join our discussions on GitHub Discussions
- Check the [documentation](https://docs.ftl.dev)
- Ask questions in issues (label with "question")

## Recognition

Contributors will be recognized in:
- The CONTRIBUTORS.md file
- Release notes for significant contributions
- The project README for major features

Thank you for contributing to FTL! 🚀