# Getting Started

This guide will walk you through creating, building, and deploying your first MCP component with FTL.

## Prerequisites

### System Requirements

- **Operating System**: macOS, Linux, or Windows (with WSL2 recommended)
- **Memory**: 4GB RAM minimum, 8GB recommended
- **Disk Space**: 2GB free space for toolchain and dependencies

### Required Software

#### 1. Rust Toolchain
FTL is written in Rust and requires the Rust toolchain:

<details>
<summary><b>macOS Installation</b></summary>

```bash
# Using Homebrew (recommended)
brew install rust

# Or using rustup directly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
</details>

<details>
<summary><b>Linux Installation</b></summary>

```bash
# Install build dependencies first
# Ubuntu/Debian:
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Fedora/RHEL:
sudo dnf install gcc gcc-c++ openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
</details>

<details>
<summary><b>Windows Installation</b></summary>

Option 1: **Windows Subsystem for Linux (WSL2)** - Recommended
```powershell
# Install WSL2
wsl --install

# Then follow Linux instructions inside WSL2
```

Option 2: **Native Windows**
1. Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Download and run [rustup-init.exe](https://rustup.rs/)
3. Follow the installation prompts
</details>

#### 2. Node.js (for TypeScript/JavaScript tools)

<details>
<summary><b>Installation Instructions</b></summary>

**macOS**:
```bash
# Using Homebrew
brew install node

# Or using nvm (Node Version Manager)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

**Linux**:
```bash
# Using NodeSource repository (Ubuntu/Debian)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Or using nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
```

**Windows**:
- Download from [nodejs.org](https://nodejs.org/)
- Or use [Chocolatey](https://chocolatey.org/): `choco install nodejs`
</details>

### Optional Tools

- **[cargo-binstall](https://github.com/cargo-bins/cargo-binstall)** - Faster binary installation
  ```bash
  cargo install cargo-binstall
  ```

- **[wkg](https://github.com/bytecodealliance/wasm-pkg-tools)** - For publishing components
  ```bash
  cargo install wkg
  ```

### Verify Installation

Run these commands to verify your setup:

```bash
# Check Rust
rustc --version  # Should show 1.87.0 or higher
cargo --version

# Check Node.js (if using TypeScript)
node --version   # Should show v20.0.0 or higher
npm --version

# Check FTL will work
which cargo      # Should show cargo path
```

## 1. Install the FTL CLI

### Quick Install

```bash
# Using cargo (will compile from source)
cargo install ftl-cli

# Using cargo-binstall (downloads pre-built binary if available)
cargo binstall ftl-cli
```

### Troubleshooting Installation

If installation fails:

1. **Update Rust**:
   ```bash
   rustup update stable
   ```

2. **Clear cargo cache**:
   ```bash
   rm -rf ~/.cargo/registry/cache
   cargo clean
   ```

3. **Install with locked dependencies**:
   ```bash
   cargo install ftl-cli --locked
   ```

4. **Check for platform-specific issues**:
   - macOS: Ensure Xcode Command Line Tools are installed: `xcode-select --install`
   - Linux: Install development packages (see Prerequisites)
   - Windows: Use WSL2 or ensure Visual Studio Build Tools are installed

### Verify Installation

```bash
ftl --version
# Should output: ftl 0.0.20 or higher
```

## 2. Create a New Project

Start by creating a new MCP project:

```bash
ftl init my-assistant
cd my-assistant
```

This creates an empty Spin project ready for adding components.

## 3. Add Your First Component

Now add a component to your project:

```bash
ftl add weather-tool --language typescript --description "Weather information for AI agents"
```

This creates:
- `weather-tool/` - Component directory
- `weather-tool/ftl.toml` - Component configuration
- `weather-tool/Makefile` - Build automation
- `weather-tool/src/` - Component source code
- Updates `spin.toml` to include the component

## 4. Implement Your Component

Edit the component implementation in `weather-tool/src/`:

### TypeScript Example

```typescript
// weather-tool/src/features.ts
import { createTool } from 'ftl-mcp';

export const tools = [
    createTool({
        name: 'get_weather',
        description: 'Get current weather for a location',
        inputSchema: {
            type: 'object',
            properties: {
                location: { type: 'string', description: 'City name' }
            },
            required: ['location']
        },
        async execute(args) {
            return `The weather in ${args.location} is sunny and 72°F`;
        }
    })
];

export const resources = [];
export const prompts = [];
```

### Rust Example

```rust
// weather-tool/src/features.rs
use ftl-mcp::*;

pub fn get_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "get_weather",
            "Get current weather for a location",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string", "description": "City name" }
                },
                "required": ["location"]
            }),
            |args| {
                let location = args["location"].as_str().unwrap_or("unknown");
                Ok(format!("The weather in {} is sunny and 72°F", location))
            }
        )
    ]
}
```

## 5. Build Your Components

```bash
ftl build
```

This compiles all components in your project into optimized WebAssembly modules.

## 6. Test Locally

Run your project locally with automatic rebuilds:

```bash
ftl watch
```

Test it with a curl request:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_weather",
      "arguments": {"location": "San Francisco"}
    },
    "id": 1
  }'
```

## 7. Publish Your Components

Publish components to a container registry:

```bash
# Publish to GitHub Container Registry (default)
ftl publish --tag v1.0.0

# Or publish to Docker Hub
ftl publish --registry docker.io --tag latest
```

Your components are now available at:
- `ghcr.io/[username]/weather-tool:v1.0.0`

## 8. Add More Components

Add additional components to your project:

```bash
# Add more components
ftl add news-tool --language typescript
ftl add calculator --language rust

# Run the project with all components
ftl watch
```

## 9. Deploy to Production

Deploy your project to FTL:

```bash
ftl deploy
```

## Next Steps

- Read the [Component Development Guide](./developing-tools.md)
- Learn about [Publishing Components](./publishing.md)
- Explore [Project Composition](./composition.md)
- Check the [CLI Reference](./cli-reference.md)