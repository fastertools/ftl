# FTL CLI v1.0.0 Release Notes

## 🎉 Initial Public Release

We're excited to announce the first public release of FTL CLI - the complete toolkit for building, composing, and deploying AI tools on WebAssembly.

## 🚀 What's New

### spin-compose (Go)
A powerful Infrastructure as Code tool for WebAssembly applications:
- **Native CUE Integration**: Direct use of CUE Go API for validation and synthesis
- **Embedded Schemas**: All schemas bundled in the binary - zero external dependencies
- **MCP Construct**: Production-ready Model Context Protocol application pattern
- **Beautiful CLI**: Intuitive, colorful interface using Cobra
- **Multi-format Support**: Accept YAML, JSON, TOML, or native CUE configurations

### FTL Platform CLI (Rust)  
Core platform management tool:
- **Project Management**: Initialize, build, test, and deploy FTL applications
- **Authentication**: Built-in OAuth/JWT support with secure token management
- **Component Management**: Add, remove, and configure WebAssembly components
- **Multi-language SDKs**: Python, Rust, TypeScript, Go support out of the box

### Pre-built Components
- **MCP Gateway**: High-performance tool routing and protocol handling
- **MCP Authorizer**: Secure authentication and authorization layer
- Both components production-tested and optimized for sub-millisecond cold starts

## 📦 Installation

```bash
# Clone and build everything
git clone https://github.com/fastertools/ftl-cli.git
cd ftl-cli
make all

# Or install individually
make build-ftl          # Build FTL CLI
make build-spin-compose # Build spin-compose
make build-components   # Build WebAssembly components
```

## 🎯 Quick Start

### Using spin-compose
```bash
# Initialize an MCP application
spin-compose init my-app --template mcp
cd my-app

# Configure your application
vim spinc.yaml

# Generate Spin manifest
spin-compose synth

# Run locally
spin up
```

### Using FTL CLI
```bash
# Initialize a project
ftl init my-project

# Add a component
ftl component add my-tool --language python

# Deploy to FTL Engine
ftl deploy
```

## 🏗️ Architecture

This monorepo contains:
- `go/spin-compose/` - Go-based composition tool using native CUE
- `cli/` - Rust-based FTL platform CLI
- `components/` - Pre-built MCP gateway and authorizer
- `sdk/` - Multi-language SDKs for building tools
- `templates/` - Quick-start templates

## 🌟 Key Features

- **Polyglot by Design**: Write tools in any language
- **Secure by Default**: WebAssembly sandboxing, authenticated endpoints
- **Production Ready**: Sub-millisecond cold starts, near-native performance
- **Open Standards**: Built on WebAssembly, Component Model, and Spin
- **No Vendor Lock-in**: Everything works without the FTL platform

## 🔮 What's Next

- Additional constructs (WordPress, microservices, AI pipelines)
- More language SDKs (Java, C#, Swift)
- Enhanced component library
- Spin community integration
- Cloud deployment options

## 🙏 Acknowledgments

Built on the shoulders of giants:
- [Spin](https://github.com/fermyontech/spin) by Fermyon
- [CUE](https://cuelang.org/) configuration language
- [WebAssembly](https://webassembly.org/) and the Component Model
- [Model Context Protocol](https://modelcontextprotocol.io/) specification

## 📚 Documentation

- [Vision](VISION.md) - Our north star and long-term goals
- [Building](BUILD.md) - Compilation and development instructions
- [Contributing](CONTRIBUTING.md) - How to contribute
- [Examples](examples/) - Sample applications and patterns

## 🐛 Known Issues

This is our v1.0.0 release. Please report issues at:
https://github.com/fastertools/ftl-cli/issues

## 📝 License

Apache 2.0 - See LICENSE file for details

---

**The Rails for AI Tools** - Making AI tool development accessible, secure, and delightful for developers everywhere.