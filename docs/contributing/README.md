# Contributing

Welcome to the FTL community! We're excited to have you contribute to building the future of polyglot AI tools. This section will help you get started with contributing to FTL.

## Ways to Contribute

### 🐛 [Report Issues](https://github.com/fastertools/ftl/issues)
Found a bug or have a feature request? Let us know:
- **Bug reports**: Clear reproduction steps, environment details
- **Feature requests**: Use cases, problem description, proposed solution
- **Documentation issues**: Unclear explanations, missing examples

### 💻 **Code Contributions**
Help improve FTL's core functionality:
- **Bug fixes**: Fix issues in CLI, SDKs, or components
- **New features**: Implement requested functionality
- **Performance improvements**: Optimize build times, runtime performance
- **SDK enhancements**: Add features to language SDKs

### 📚 **Documentation**
Improve FTL's documentation:
- **Tutorials**: New getting-started guides and walkthroughs
- **Examples**: Real-world use cases and implementation patterns
- **API docs**: Better explanations and code examples
- **Translations**: Documentation in other languages

### 🔧 **Tools and Infrastructure**
Enhance the developer experience:
- **CI/CD improvements**: Faster, more reliable builds
- **Testing**: Better test coverage, integration tests
- **Developer tooling**: IDE plugins, debugging tools
- **Templates**: Project scaffolding for new languages

## Getting Started

### [Development Setup](./development-setup.md)
Learn how to set up your development environment:
- **Build FTL from source** - Complete setup guide
- **Development workflow** - Testing, building, running locally
- **IDE configuration** - Recommended settings and extensions

### [Contribution Process](./contribution-process.md)
Understand our development process:
- **Pull request workflow** - From fork to merge
- **Code review process** - What reviewers look for
- **Issue triage** - How we prioritize and assign work
- **Release process** - How changes make it to users

### [Code of Conduct](./code-of-conduct.md)
Our community guidelines:
- **Expected behavior** - How we interact with each other
- **Unacceptable behavior** - What crosses the line
- **Reporting process** - How to report violations
- **Enforcement** - How we handle violations

### [Architecture for Contributors](./architecture.md)
Technical deep dive for core contributors:
- **Codebase organization** - How the code is structured
- **Design decisions** - Why things work the way they do
- **Adding new features** - Patterns and best practices
- **Testing strategy** - How to test your changes

## Quick Contribution Guide

### 1. **Find Something to Work On**
- Browse [good first issues](https://github.com/fastertools/ftl/labels/good%20first%20issue)
- Check [help wanted](https://github.com/fastertools/ftl/labels/help%20wanted) issues
- Look at our [roadmap](https://github.com/fastertools/ftl/projects) for larger projects
- Propose your own improvements

### 2. **Set Up Development Environment**
```bash
# Clone the repository
git clone https://github.com/fastertools/ftl.git
cd ftl

# Follow the development setup guide
# See: docs/contributing/development-setup.md
```

### 3. **Make Your Changes**
- Create a feature branch from `main`
- Write tests for your changes
- Follow our [coding standards](#coding-standards)
- Update documentation as needed

### 4. **Submit Your Contribution**
- Open a pull request with a clear description
- Reference any related issues
- Respond to code review feedback
- Celebrate when your PR is merged! 🎉

## Coding Standards

### General Principles
- **Security first**: No unsafe code without exceptional justification
- **Performance matters**: Optimize for startup time and memory usage
- **User experience**: Simple commands, clear error messages
- **Backward compatibility**: Don't break existing projects without migration path

### Rust Code
- Follow `cargo fmt` formatting
- Pass all `cargo clippy` lints
- No `panic!()`, `unwrap()`, or `expect()` in production code
- Comprehensive error handling with context
- Document public APIs with examples

### Go Code  
- Use `gofmt` for formatting
- Pass `go vet` and `golint` checks
- Follow Go naming conventions
- Include benchmarks for performance-critical code

### Python Code
- Use `black` for code formatting  
- Pass `ruff` linting checks
- Type hints for all public functions
- Follow PEP 8 style guide
- Comprehensive docstrings

### Documentation
- Clear, concise writing
- Working code examples
- Screenshots for UI changes
- Cross-references to related topics

## Testing Requirements

All contributions must include appropriate tests:

### Unit Tests
- Test individual functions and components
- Cover both success and failure cases
- Mock external dependencies
- Fast execution (< 1 second per test)

### Integration Tests
- Test component interactions
- Use real FTL CLI commands
- Verify end-to-end workflows
- Test in CI environment

### Documentation Tests
- Ensure all code examples work
- Test installation instructions
- Verify links and references
- Check formatting and grammar

## Review Process

### What Reviewers Look For

**Functionality**:
- Does the code solve the stated problem?
- Are edge cases handled properly?
- Is error handling comprehensive?

**Code Quality**:
- Is the code readable and maintainable?
- Are tests comprehensive and reliable?
- Does it follow project conventions?

**User Experience**:
- Is the feature discoverable and intuitive?
- Are error messages helpful?
- Is documentation clear and complete?

**Compatibility**:
- Does it work across supported platforms?
- Are breaking changes properly communicated?
- Is the upgrade path clear?

### Response Time
- **Initial response**: Within 48 hours
- **Full review**: Within 1 week for most PRs
- **Complex changes**: May take longer, we'll communicate timeline

### Addressing Feedback
- Respond to all review comments
- Make requested changes or explain why not
- Ask for clarification when needed
- Be patient and professional

## Recognition

We value all contributions and want to recognize your work:

### Contributors File
All contributors are listed in [CONTRIBUTORS.md](../../CONTRIBUTORS.md)

### Release Notes
Significant contributions are highlighted in release notes

### Community Recognition
Outstanding contributors may be invited to:
- Join the maintainer team
- Speak at conferences about FTL
- Preview new features before release
- Influence roadmap decisions

## Getting Help

### For Questions
- **Discord**: [FTL Community Server](https://discord.gg/ftl) 
- **GitHub Discussions**: [github.com/fastertools/ftl/discussions](https://github.com/fastertools/ftl/discussions)
- **Email**: [contributors@ftlengine.dev](mailto:contributors@ftlengine.dev)

### For Issues
- **Bug reports**: [GitHub Issues](https://github.com/fastertools/ftl/issues)
- **Security issues**: [security@ftlengine.dev](mailto:security@ftlengine.dev)
- **Code of conduct**: [conduct@ftlengine.dev](mailto:conduct@ftlengine.dev)

## Contributor Resources

### External Tools
- **Rust**: [The Rust Book](https://doc.rust-lang.org/book/)
- **WebAssembly**: [WebAssembly Documentation](https://webassembly.org/)
- **Spin Framework**: [Fermyon Spin Docs](https://developer.fermyon.com/spin)
- **MCP Protocol**: [Model Context Protocol Spec](https://spec.modelcontextprotocol.io/)

### Learning Resources
- **Architecture Overview**: Start with [Core Concepts](../core-concepts/)
- **Code Examples**: Browse [Examples](../../examples/)
- **Testing Patterns**: See [Testing Your Tools](../guides/testing.md)
- **WebAssembly Patterns**: Learn [Why WebAssembly?](../core-concepts/why-webassembly.md)

Thank you for contributing to FTL! Your efforts help make polyglot AI tools accessible to everyone.