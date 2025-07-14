# Troubleshooting Guide

This guide helps you resolve common issues when using FTL. If you don't find your issue here, please [open an issue](https://github.com/fastertools/ftl-cli/issues) on GitHub.

## Installation Issues

### cargo install ftl-cli fails

**Symptom**: Installation via `cargo install ftl-cli` fails with compilation errors.

**Solutions**:
1. Ensure you have the latest Rust version:
   ```bash
   rustup update stable
   rustc --version  # Should be 1.87+
   ```

2. Clear cargo cache and retry:
   ```bash
   cargo clean
   rm -rf ~/.cargo/registry/cache
   cargo install ftl-cli
   ```

3. Install with locked dependencies:
   ```bash
   cargo install ftl-cli --locked
   ```

### Platform-Specific Installation

#### macOS
- Ensure Xcode Command Line Tools are installed:
  ```bash
  xcode-select --install
  ```

#### Linux
- Install build essentials:
  ```bash
  # Ubuntu/Debian
  sudo apt-get update
  sudo apt-get install build-essential pkg-config libssl-dev

  # Fedora/RHEL
  sudo dnf install gcc gcc-c++ openssl-devel
  ```

#### Windows
- Use Windows Subsystem for Linux (WSL) for best compatibility
- Or ensure Visual Studio C++ build tools are installed

## Template Setup Issues

### ftl setup templates fails

**Symptom**: Template setup fails with network or permission errors.

**Solutions**:
1. Check network connectivity:
   ```bash
   curl -I https://github.com/fastertools/ftl-mcp
   ```

2. Manually specify template source:
   ```bash
   ftl setup templates --git https://github.com/fastertools/ftl-mcp --branch main
   ```

3. Use a local template directory:
   ```bash
   git clone https://github.com/fastertools/ftl-mcp ~/ftl-templates
   ftl setup templates --dir ~/ftl-templates
   ```

## Build Errors

### WebAssembly target not found

**Symptom**: Build fails with "can't find crate for `std`" or target errors.

**Solution**:
```bash
# Install WebAssembly target
rustup target add wasm32-wasip1

# For older Rust versions
rustup target add wasm32-wasi
```

### TypeScript build failures

**Symptom**: TypeScript tools fail to build with module errors.

**Solutions**:
1. Ensure Node.js 20+ is installed:
   ```bash
   node --version  # Should be v20.0.0 or higher
   ```

2. Clear npm cache:
   ```bash
   npm cache clean --force
   cd your-tool && rm -rf node_modules package-lock.json
   npm install
   ```

3. Check ftl-sdk version compatibility:
   ```json
   {
     "dependencies": {
       "ftl-sdk": "^0.2.0"
     }
   }
   ```

### cargo-component not found

**Symptom**: Rust builds fail with "cargo-component: command not found".

**Solution**: FTL should auto-install cargo-component. If it doesn't:
```bash
cargo install cargo-component
```

## Runtime Errors

### ftl up fails to start

**Symptom**: Server fails to start or immediately exits.

**Solutions**:
1. Check port availability:
   ```bash
   lsof -i :3000  # Default port
   # Or use a different port
   ftl up --listen 127.0.0.1:8080
   ```

2. Verify spin.toml exists:
   ```bash
   ls spin.toml
   # If missing, you're not in a project directory
   ```

3. Check Spin installation:
   ```bash
   spin --version
   # If missing, FTL will prompt to install
   ```

### Tool not found errors

**Symptom**: MCP gateway can't find your tools.

**Solutions**:
1. Verify tool registration in spin.toml:
   ```toml
   [component.mcp-gateway.variables]
   tool_components = ["my-tool", "another-tool"]
   ```

2. Check tool naming (must be kebab-case):
   ```bash
   # Correct
   ftl add my-tool
   
   # Incorrect (will be converted)
   ftl add my_tool  # Becomes my-tool
   ```

3. Rebuild after adding tools:
   ```bash
   ftl build
   ftl up
   ```

### Component communication failures

**Symptom**: Tools can't communicate with each other.

**Solution**: Use Spin's internal networking:
```typescript
// Correct - uses internal component name
const response = await fetch('http://my-tool.spin.internal/');

// Incorrect - uses external URL
const response = await fetch('http://localhost:3000/my-tool');
```

## Development Issues

### Hot reload not working

**Symptom**: `ftl watch` doesn't detect file changes.

**Solutions**:
1. Check file permissions
2. Verify watch patterns in your tool's config
3. Try manual rebuild:
   ```bash
   ftl build && ftl up
   ```

### Test failures

**Symptom**: `ftl test` reports failures.

**Solutions**:
1. Run tests with verbose output:
   ```bash
   ftl test -- --nocapture  # For Rust
   ftl test -- --verbose    # For TypeScript
   ```

2. Test individual components:
   ```bash
   cd my-tool && cargo test      # Rust
   cd my-tool && npm test        # TypeScript
   ```

## Deployment Issues

### Deploy command fails

**Symptom**: `ftl deploy` fails with authentication or network errors.

**Solutions**:
1. Check Fermyon Cloud authentication:
   ```bash
   spin login
   ```

2. Verify deployment configuration in spin.toml

3. Try direct Spin deployment:
   ```bash
   spin deploy
   ```

### Large bundle sizes

**Symptom**: Deployment fails due to size limits.

**Solutions**:
1. Build with release optimizations:
   ```bash
   ftl build --release
   ```

2. Check component sizes:
   ```bash
   ls -lh target/wasm32-wasip1/release/*.wasm
   ```

3. See [Performance Guide](./performance.md) for optimization tips

## Common Error Messages

### "No such file or directory"
- Ensure you're in the project root (contains spin.toml)
- Check file paths are correct
- Verify templates are installed: `ftl setup templates`

### "Failed to validate component"
- Check your tool implements required MCP methods
- Verify JSON schema is valid
- Ensure handler returns proper ToolResponse

### "Permission denied"
- Check file ownership: `ls -la`
- Ensure write permissions in project directory
- On Unix: `chmod -R u+w .`

### "Network error"
- Check internet connectivity
- Verify proxy settings if behind corporate firewall
- Try using different DNS: `8.8.8.8`

## Debug Mode

Enable debug logging for more information:

```bash
# Set log level
export RUST_LOG=debug
ftl build

# Verbose output
ftl up --verbose

# Component-specific debugging
export SPIN_LOG=trace
ftl up
```

## Getting Help

If you're still experiencing issues:

1. **Search existing issues**: [GitHub Issues](https://github.com/fastertools/ftl-cli/issues)
2. **Ask in discussions**: [GitHub Discussions](https://github.com/fastertools/ftl-cli/discussions)
3. **File a bug report**: Include:
   - FTL version: `ftl --version`
   - Rust version: `rustc --version`
   - Platform: macOS/Linux/Windows
   - Error messages and logs
   - Steps to reproduce

## FAQ

**Q: Why WebAssembly?**
A: WebAssembly provides sandboxed execution, predictable performance, and polyglot support.

**Q: Can I use FTL without Spin?**
A: No, FTL is built on top of Spin's WebAssembly runtime.

**Q: How do I update FTL?**
A: Run `cargo install ftl-cli --force`

**Q: Where are templates stored?**
A: Templates are managed by Spin in `~/.spin/templates/`

**Q: Can I use custom templates?**
A: Yes, use `--git`, `--dir`, or `--tar` options with `ftl setup templates`