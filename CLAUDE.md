# FTL CLI Development Constraints

## WASM Runtime Constraints

**CRITICAL**: This project runs in WebAssembly (WASM) environments with significant runtime limitations.

### Forbidden Dependencies during development until explicitly approved
- L `tokio` - Not supported in WASM, uses system threads
- L `async-trait` - Relies on tokio/futures runtime  
- L `futures` crate - Runtime dependencies not available
- L Any dependency requiring thread spawning or OS-level async runtime

### WASM-Compatible Async
- Native `async/await` syntax - Works in WASM
- `spin-sdk` async functions - Spin runtime provides async support
- Simple async functions without external runtimes
- `ftl-sdk` async tools - Built for WASM compatibility

### Testing in WASM
- **NEVER** use `#[tokio::test]` for async tests - not supported in WASM  
- Standard Rust tests (`#[test]`) cannot be async functions
- Use `#[test]` for sync-only unit tests
- Async functionality should be tested in integration tests or tools
- For async testing, create the async context within sync test functions
- Spin provides the async runtime, not tokio

### Architecture Implications
- All async operations must use the Spin runtime's native async support
- No external async runtimes or thread pools
- No tokio::time or other tokio utilities
- No async-trait macro - use manual async trait implementations
- Keep async simple and runtime-agnostic

### When Adding Dependencies
1. Check if dependency works in `no_std` or WASM environments
2. Avoid anything requiring `std::thread` or system-level async
3. Test compilation with `cargo check --target wasm32-wasi` if unsure
4. Prefer pure Rust implementations over system-dependent crates

This constraint applies to ALL components in this project.

## Templ Template Engine - JavaScript/Hover Functionality

**CRITICAL**: This project uses the templ template engine for type-safe HTML generation. Here are the correct patterns for implementing hover functionality and JavaScript interactions.

### Script Template Syntax (Recommended)

For onmouseover, onmouseout, and other JavaScript event handlers, use templ script templates:

```go
script handleHover(bgColor, borderColor string) {
    this.style.backgroundColor = bgColor;
    this.style.borderColor = borderColor;
}

script handleHoverOut(bgColor, borderColor string) {
    this.style.backgroundColor = bgColor;
    this.style.borderColor = borderColor;
}

templ MyButton() {
    <button 
        onmouseover={ handleHover("#5a5a5a", "#6a6a6a") }
        onmouseout={ handleHoverOut("#4a4a4a", "#5a5a5a") }
        style="background-color: #4a4a4a; border: 1px solid #5a5a5a;"
    >
        Click me
    </button>
}
```

### Key Rules for Templ JavaScript

1. **USE script templates** - Don't use `fmt.Sprintf` for JavaScript event handlers
2. **Parameters are properly sanitized** - templ handles injection protection
3. **Functions are unique** - templ generates collision-free function names
4. **Type safety** - Go compiler validates script parameters

### What NOT to do (Common Mistakes)

```go
// ❌ WRONG - This causes compilation errors
onmouseover={ fmt.Sprintf("this.style.backgroundColor='%s'", color) }

// ❌ WRONG - Raw JavaScript strings are not type-safe
onmouseover="this.style.backgroundColor='#5a5a5a'"

// ❌ WRONG - Cannot use Go expressions in JavaScript attributes directly
style={ fmt.Sprintf("color: %s;", ui.Colors.UI["text"]) }
```

### Correct Patterns

```go
// ✅ CORRECT - Use script templates for event handlers
script changeColor(color string) {
    this.style.backgroundColor = color;
}

// ✅ CORRECT - Use Go expressions for CSS styles  
style={ fmt.Sprintf("color: %s;", ui.Colors.UI["text"]) }

// ✅ CORRECT - Combine both approaches
templ Button() {
    <button 
        onmouseover={ changeColor("#5a5a5a") }
        onmouseout={ changeColor("#4a4a4a") }
        style={ fmt.Sprintf("background-color: %s;", ui.Colors.UI["button_bg"]) }
    >
        Button
    </button>
}
```

### Legacy vs Modern

- **Script templates**: Recommended for simple interactions, but considered legacy
- **Modern approach**: Use `templ.JSFuncCall`, `templ.JSONString`, and bundlers like esbuild for complex apps
- **This project**: Use script templates for simplicity unless complexity requires modern approach

## README Template Standards

### Standard Structure for All SDK Templates

To ensure consistency across all FTL SDK templates (Rust, Python, etc.), use this standardized README structure:

```markdown
# {{project-name}}

An FTL MCP tool written in [Rust/Python/Go/etc].

## Prerequisites
[SDK-specific requirements - e.g., Rust 1.86+, Python 3.10+, Go 1.21+]

## Quick Start
1. Build steps (SDK-specific)
2. Run steps (consistent: `ftl build && ftl up`)

## Platform-Specific Notes
[Include if needed - e.g., Windows setup differences]

## Development
### Project Structure
[Show file tree with SDK-specific structure]

### Available Commands
[SDK-specific make/cargo commands]

### Adding New Tools
[SDK-specific examples showing how to add tools]

### Testing
[SDK-specific test examples and commands]

### Code Quality
[SDK-specific linting/formatting tools - Clippy+rustfmt for Rust, Black+Ruff+MyPy for Python]

## Deployment
```bash
ftl eng deploy
```
```

### Implementation Notes
- Each SDK template should have a comprehensive README.md in its content/ directory
- Use template variables like `{{project-name}}` for dynamic content
- Keep deployment section consistent across all SDKs
- Ensure examples are runnable and tested
- Include troubleshooting sections for common issues