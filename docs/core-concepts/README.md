# Core Concepts

Understanding how FTL works under the hood will help you build better tools and debug issues more effectively. This section explains the key concepts and architecture that make FTL's polyglot capabilities possible.

## Concepts

### [Why WebAssembly?](./why-webassembly.md)
**The foundation of FTL's capabilities**  
Learn why WebAssembly was chosen as the execution runtime and how it enables speed, security, and portability for AI tools.

### [The Component Model](./component-model.md)
**How language interoperability works**  
Understand the WebAssembly Component Model that allows tools written in different languages to work together seamlessly.

### [FTL Architecture](./architecture.md)
**Deep dive into system design**  
Explore the complete FTL architecture, from MCP clients through the gateway and authorizer to your tool components.

## Learning Path

If you're new to these concepts, we recommend reading them in order:

1. **Start with [Why WebAssembly?](./why-webassembly.md)** - Understand the foundational choice
2. **Then read [The Component Model](./component-model.md)** - Learn how languages connect
3. **Continue with [FTL Architecture](./architecture.md)** - See the complete system


## Key Principles

Understanding these principles will help you work more effectively with FTL:

- **Security First**: Every tool runs in an isolated WebAssembly sandbox
- **Language Agnostic**: The Component Model provides universal interoperability
- **Standards-Based**: Built on MCP, WASM, and other open standards
- **Developer Experience**: Complex infrastructure hidden behind simple commands
- **Production Ready**: Designed for real-world deployment and scaling

## Next Steps

After understanding these concepts:
- Implement advanced patterns from [Examples](../../examples/README.md)
- Reference specific APIs in [SDKs](../../sdk/README.md)