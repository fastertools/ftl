# fastertools FTL Documentation

Welcome to the official documentation for fastertools CLI and framework!

Whether you're building your first AI tool or already deploying a complex polyglot server to the edge, these docs are your guide to mastering the framework. The docs are organized into the general categories that should help best answer the questions you have.

These are living documents. If you see something that could be improved, please let us know or cut a PR—leave it better than you found it!

## Documentation Structure

### [Getting Started](./getting-started/)
**Tutorials for absolute beginners**
- [Your First FTL Project](./getting-started/first-project.md) - Step-by-step tutorial from init to running tools
- [Composing Polyglot Servers](./getting-started/polyglot-composition.md) - Add tools in different languages

### [Core Concepts](./core-concepts/)
**Understanding how FTL works**
- [Why WebAssembly?](./core-concepts/why-webassembly.md) - The benefits of WASM for AI tools
- [Component Model](./core-concepts/component-model.md) - How language interoperability works
- [FTL Architecture](./core-concepts/architecture.md) - Deep dive into system design
- [Project Lifecycle](./core-concepts/lifecycle.md) - What happens behind the commands

### [How-to Guides](./guides/)
**Goal-oriented recipes for specific problems**
- [Making HTTP Requests](./guides/http-requests.md) - Call third-party APIs from tools
- [Handling Authentication](./guides/authentication.md) - OAuth with mcp-authorizer
- [Testing Your Tools](./guides/testing.md) - Write and run tests

### [SDK Reference](./sdk-reference/)
**Technical API documentation**
- [Rust SDK](./sdk-reference/rust/) - Complete Rust API reference
- [Python SDK](./sdk-reference/python/) - Complete Python API reference
- [Go SDK](./sdk-reference/go/) - Complete Go API reference
- [TypeScript SDK](./sdk-reference/typescript/) - Complete TypeScript API reference

### [Contributing](./contributing/)
**Join the FTL community**
- [Code of Conduct](./contributing/code-of-conduct.md) - Community guidelines
- [Development Setup](./contributing/development-setup.md) - Build FTL from source
- [Contribution Process](./contributing/contribution-process.md) - PRs and issue workflow
- [Architecture for Contributors](./contributing/architecture.md) - Technical deep dive

## Additional Resources

### FTL Project Resources
- [Examples](../examples/) - Complete example projects
- [Templates](../templates/) - Project scaffolding templates

### WebAssembly & Component Model
- [WebAssembly.org](https://webassembly.org/) - Official WebAssembly documentation and specifications
- [Component Model](https://component-model.bytecodealliance.org/) - WebAssembly Component Model specification and guides
- [WASI](https://wasi.dev/) - WebAssembly System Interface for secure system access

### Spin Framework
- [Spin Documentation](https://developer.fermyon.com/spin) - Complete Fermyon Spin framework documentation
- [Spin WebAssembly Functions](https://developer.fermyon.com/spin/v2/writing-apps) - Guide to writing WebAssembly applications with Spin
- [Spin Templates](https://developer.fermyon.com/spin/v2/quickstart) - Official Spin project templates and quickstarts
- [Spin SDK Reference](https://developer.fermyon.com/spin/v2/rust-components) - Language-specific SDK documentation

### Related Technologies  
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) - The protocol FTL implements for AI tool communication
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification) - The RPC protocol used by MCP
- [OpenAPI/JSON Schema](https://json-schema.org/) - Schema format for tool input validation