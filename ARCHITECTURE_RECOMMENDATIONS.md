# FTL CLI Architecture Recommendations

## Executive Summary

After analyzing the new template system and existing CLI implementation, I recommend a significant architectural shift to align FTL with the new component-based MCP server model. The key change is moving from managing full Spin applications to managing individual MCP components that can be composed into Spin applications.

## Current State Analysis

### New Template System
- Templates now generate **standalone WebAssembly components** implementing the MCP protocol
- Each component is designed to be published to a registry (e.g., `ghcr.io/username/component:version`)
- Components use a pre-published `mcp-gateway` component for HTTP->MCP translation
- Templates include Makefiles with `registry-push` targets for publishing

### SDK Architecture
- Three SDKs (Rust, TypeScript, JavaScript) provide consistent APIs for building MCP handlers
- SDKs handle tool registration, resource management, and prompt handling
- Clean separation between MCP protocol implementation and user code

### Existing CLI Issues
1. **Tool-centric model** - Commands like `ftl new`, `ftl build`, `ftl deploy` assume managing complete tools
2. **Toolkit abstraction** - The toolkit concept doesn't align with Spin's component composition model
3. **Custom spin.toml generation** - Overly complex for the new component model
4. **Missing component lifecycle** - No support for publishing/versioning components

## Recommended Architecture

### 1. Embrace Component-First Design

**Remove the "tool" abstraction entirely.** Instead, work with:
- **MCP Components**: Individual WebAssembly components implementing MCP features
- **Spin Projects**: Compositions of multiple MCP components

### 2. New CLI Command Structure

```bash
# Component Development
ftl init <name>                    # Create new MCP component project
ftl dev                            # Run component in development mode
ftl test                           # Test the component
ftl build                          # Build the component
ftl publish                        # Publish to registry

# Component & Project Management  
ftl init <name>                       # Create empty Spin project (wraps spin new -t http-empty)
ftl add <component>                   # Add component to current project
ftl up                                # Serve the project locally
ftl deploy                            # Deploy to FTL

# Component Management
ftl registry list                  # List available components
ftl registry search <query>        # Search for components
ftl registry info <component>      # Show component details
```

### 3. Simplified Workflow

**For single component development:**
```bash
ftl init my-weather-tool --lang typescript
cd my-weather-tool
ftl dev                           # Starts at localhost:3000/mcp
ftl publish                       # Publishes to registry
```

**For multi-component projects:**
```bash
ftl init my-assistant
cd my-assistant
ftl add github-tool --language typescript
ftl add weather-tool --language typescript
ftl add my-custom-tool --language rust
ftl up                                # All components available
ftl deploy
```

### 4. Key Implementation Changes

#### Remove Complexity
1. **Delete toolkit functionality** - It's redundant with Spin's component model
2. **Simplify manifest handling** - Use ftl.toml only for component metadata
3. **Remove custom spin.toml generation** - Let Spin handle this via templates

#### Add Component Features
1. **Registry integration** - First-class support for publishing/consuming components
2. **Version management** - Handle component versioning properly
3. **Dependency resolution** - Ensure compatible component versions

#### Improve Developer Experience
1. **Template installation** - Auto-install templates on first use
2. **Component scaffolding** - Quick generators for common patterns
3. **Testing utilities** - Built-in MCP protocol testing

### 5. Migration Path

1. **Phase 1**: Add new commands alongside existing ones
2. **Phase 2**: Deprecate old commands with migration guides
3. **Phase 3**: Remove deprecated commands in next major version

### 6. Technical Details

#### Component Publishing
- Use `wkg` (WebAssembly package manager) for registry operations
- Support multiple registries (GitHub Container Registry, Docker Hub, etc.)
- Include metadata in published components (description, version, MCP features)

#### Local Development
- Use Spin's built-in development server
- Provide MCP-specific development tools (request inspection, mock clients)
- Hot reload support where possible

#### App Composition
- Leverage Spin's component composition features
- Support both local and registry components
- Handle component dependency conflicts

## Benefits

1. **Simplicity** - Aligns with Spin's model instead of fighting it
2. **Modularity** - True component reusability across projects
3. **Ecosystem** - Enables component marketplace/discovery
4. **Standards** - Follows WebAssembly component model standards
5. **Future-proof** - Ready for WASI Preview 2 and component model evolution

## Risks and Mitigations

1. **Breaking changes** - Mitigate with careful migration path and tooling
2. **Learning curve** - Provide excellent documentation and examples
3. **Registry dependency** - Support local-only workflows as fallback

## Next Steps

1. Validate approach with key stakeholders
2. Create proof-of-concept for new command structure
3. Design component registry integration
4. Plan migration strategy for existing users

## Conclusion

This architecture positions FTL as a best-in-class tool for building and deploying MCP servers on WebAssembly. By embracing Spin's component model rather than abstracting over it, we can provide a simpler, more powerful developer experience while enabling a rich ecosystem of reusable MCP components.