# FTL Vision: The Rails for AI Tools

## Mission
Enable anyone in any language to easily author, manage, deploy, and run secure high-performance MCP servers on WebAssembly.

## The Big Picture

We're building the **definitive platform for AI tool infrastructure** - making MCP servers as easy to create and deploy as web apps. Everything we build is open source and valuable independent of the platform itself, following the Spin/Fermyon model.

### Architecture

```
ftl-cli (monorepo)
├── spin-compose (Go)         # Universal Spin composition
├── ftl (Rust)                # Platform orchestration  
├── components/               # MCP gateway, authorizer
├── sdk/                      # Multi-language SDKs
└── templates/                # Quick-start patterns
```

### Why This Architecture Wins

1. **Polyglot by Design**
   - SDKs in every language
   - Components in any language  
   - Tools in the best language for each job
   - United by WebAssembly

2. **Open Source Value Stack**
   - `spin-compose` → Valuable to all Spin users
   - MCP components → Valuable to all MCP users
   - SDKs → Valuable to all WASM developers
   - FTL platform → Ties it together with deployment

3. **AI-First Infrastructure**
   - MCP is becoming the standard for AI tools
   - WebAssembly provides security isolation
   - Spin provides the runtime
   - FTL provides the platform

## The Developer Journey

1. **Choose Your Language** - Python, Rust, Go, TypeScript, etc.
2. **Use Our SDK** - Simple, idiomatic APIs for each language
3. **Define in spinc.yaml** - High-level, declarative configuration
4. **Test Locally** - Fast iteration with hot reload
5. **Deploy with FTL** - One command to production

## Strategic Advantages

### Lower Barrier to Entry
- Start with templates
- Use pre-built components
- Graduate to custom code
- Deploy anywhere

### Network Effects
- More components → More useful platform
- More languages → More developers
- More tools → More AI capabilities
- Community contributions benefit everyone

### Not Just Another Platform
- Everything works without FTL
- Value at every layer
- Community-first approach
- No vendor lock-in

## Core Principles

1. **Convention over Configuration** - Smart defaults, minimal boilerplate
2. **Batteries Included** - Everything you need out of the box
3. **Secure by Default** - WebAssembly isolation, authenticated by default
4. **Performance First** - Near-native speed, minimal overhead
5. **Truly Polyglot** - First-class support for all languages

## Implementation Priorities

1. **spin-compose (Go)** - The foundation for declarative Spin apps
2. **SDK Excellence** - Make building components delightful
3. **Component Library** - Rich ecosystem of pre-built MCP tools
4. **Templates** - Quick starts for common patterns
5. **FTL Platform** - Seamless deployment and management

## The Vision

We're not just building tools - we're enabling the next generation of AI applications. As AI agents become more capable, they need:

- **Secure execution environments** (WebAssembly)
- **Standard protocols** (MCP)
- **Easy deployment** (FTL)
- **Any language** (Polyglot SDKs)
- **Composable tools** (spin-compose)

By making it trivially easy to create and deploy MCP servers, we're accelerating AI progress. Every developer, in any language, can contribute tools that AI agents can use.

## Success Metrics

- Developers can go from zero to deployed MCP server in under 5 minutes
- Supporting 10+ languages with idiomatic SDKs
- Hundreds of pre-built components in the ecosystem
- Adopted by major AI platforms as the standard for tool deployment
- spin-compose becomes the de facto standard for Spin composition

## Open Source Philosophy

Like Spin with Fermyon, we're building in the open:
- spin-compose is useful without FTL
- SDKs work with any Spin runtime
- Components run anywhere Spin runs
- Documentation is comprehensive and free
- Community contributions are celebrated

This is our north star. Every decision should move us closer to making AI tool development accessible, secure, and delightful for developers everywhere.