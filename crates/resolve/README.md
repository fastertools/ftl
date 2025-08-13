# ftl-resolve

Component resolver and transpiler for FTL configuration files. Downloads registry components using wkg and transpiles to Spin TOML format.

## Installation

### From Crates.io (once published)
```bash
cargo install ftl-resolve
```

### From Source
```bash
cargo build --release --package ftl-resolve
sudo cp target/release/ftl-resolve /usr/local/bin/

# Optional: Install man page
sudo cp target/man/ftl-resolve.1 /usr/local/share/man/man1/
sudo gzip -f /usr/local/share/man/man1/ftl-resolve.1
```

### Binary Installation
```bash
# Linux (x86_64)
wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-linux-amd64
sudo install -m 755 ftl-resolve-linux-amd64 /usr/local/bin/ftl-resolve

# macOS (Apple Silicon)
wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-darwin-arm64
sudo install -m 755 ftl-resolve-darwin-arm64 /usr/local/bin/ftl-resolve

# macOS (Intel)
wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-darwin-amd64
sudo install -m 755 ftl-resolve-darwin-amd64 /usr/local/bin/ftl-resolve

# Optional: Download and install man page
wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve.1.gz
sudo install -m 644 ftl-resolve.1.gz /usr/local/share/man/man1/
```

### Prerequisites

ftl-resolve requires `wkg` for resolving registry components:
```bash
# Install wkg
cargo install wkg
```

## Usage

### Generate Spin TOML

By default, ftl-resolve uses wkg to download and resolve registry components:
```bash
# Generate spin.toml with wkg resolution (default)
ftl-resolve spin ftl.toml -o spin.toml

# From stdin to stdout
cat ftl.toml | ftl-resolve spin -

# From JSON input
ftl-resolve spin config.json -f json -o spin.toml

# Specify project directory for relative paths
ftl-resolve spin ftl.toml -o spin.toml -d /path/to/project
```

### Use Spin's Native Registry Resolution

Preserve registry references in Spin's format instead of downloading with wkg:
```bash
# Keep registry references for Spin to resolve
ftl-resolve spin ftl.toml -o spin.toml --spin-resolve
```

### Generate JSON Schema

Generate a JSON schema for FTL configuration validation:
```bash
# To stdout
ftl-resolve schema

# To file
ftl-resolve schema -o ftl-schema.json
```

### Validate Configuration

Validate an FTL configuration file:
```bash
# Validate TOML
ftl-resolve validate ftl.toml

# Validate JSON  
ftl-resolve validate config.json -f json

# Validate from stdin
cat ftl.toml | ftl-resolve validate -
```

## Component Resolution

ftl-resolve automatically detects and resolves registry components:

### Registry Component Format
```toml
[mcp]
gateway = "ghcr.io/fastertools/mcp-gateway:0.0.11"
authorizer = "ghcr.io/fastertools/mcp-authorizer:0.0.14"

[component.my-tool]
repo = "ghcr.io/myorg/my-tool:1.0.0"
```

### Resolution Process
1. Detects registry references (format: `registry.domain/namespace/package:version`)
2. Uses `wkg oci pull` to download components to `.ftl/wasm/` directory
3. Updates configuration with local paths
4. Transpiles to Spin TOML with resolved paths

### Docker Credentials
`wkg` uses Docker credentials for authentication:
```bash
# Login to registry
docker login ghcr.io

# wkg will use these credentials automatically
ftl-resolve resolve -i ftl.toml -o spin.toml
```

## Integration with Backend Lambda

### Shell Command Usage
```bash
# In your deployment script or Lambda
ftl-resolve spin ftl.toml -o spin.toml
spin up -f spin.toml
```

### Example Usage in Lambda
```python
import subprocess
import json
import tempfile
import os

def resolve_and_transpile(ftl_config_dict, project_dir="."):
    """
    Resolve components and transpile FTL config to Spin TOML.
    """
    # Write config as JSON to temp file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(ftl_config_dict, f)
        config_path = f.name
    
    try:
        # Run resolver
        result = subprocess.run(
            ['ftl-resolve', 'spin', config_path, '-o', 'spin.toml', '-d', project_dir],
            capture_output=True,
            text=True,
            check=True
        )
        
        # Read generated spin.toml
        with open('spin.toml', 'r') as f:
            return f.read()
    finally:
        os.unlink(config_path)
```

## Docker Integration

Add to your `Dockerfile`:
```dockerfile
# Install wkg and ftl-resolve
RUN cargo install wkg ftl-resolve

# Or use release binaries
RUN wget https://github.com/bytecodealliance/wasm-pkg-tools/releases/latest/download/wkg-linux-amd64 \
    -O /usr/local/bin/wkg && \
    chmod +x /usr/local/bin/wkg && \
    wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-linux-amd64 \
    -O /usr/local/bin/ftl-resolve && \
    chmod +x /usr/local/bin/ftl-resolve
```

### Lambda Deployment Example
```dockerfile
FROM public.ecr.aws/lambda/python:3.11

# Install wkg and ftl-resolve
RUN yum install -y wget && \
    wget https://github.com/bytecodealliance/wasm-pkg-tools/releases/latest/download/wkg-linux-amd64 \
    -O /usr/local/bin/wkg && \
    chmod +x /usr/local/bin/wkg && \
    wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-linux-amd64 \
    -O /usr/local/bin/ftl-resolve && \
    chmod +x /usr/local/bin/ftl-resolve

# Your Lambda code
COPY lambda_function.py ./
COPY requirements.txt ./
RUN pip install -r requirements.txt

CMD ["lambda_function.handler"]
```

## Why ftl-resolve?

While Spin supports registry references, ftl-resolve provides:
- **Deterministic resolution**: Components are resolved at deployment time
- **Better reliability**: Uses wkg, the WebAssembly community standard
- **Fresh pulls**: Always downloads latest version (no caching issues)
- **Explicit control**: Know exactly what components are being deployed
- **Better compatibility**: Works around Spin's incomplete registry support

## CI/CD Pipeline Integration

```yaml
# GitHub Actions example
name: Deploy FTL App
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install tools
        run: |
          wget https://github.com/bytecodealliance/wasm-pkg-tools/releases/latest/download/wkg-linux-amd64 \
            -O wkg && chmod +x wkg && sudo mv wkg /usr/local/bin/
          wget https://github.com/fastertools/ftl-cli/releases/latest/download/ftl-resolve-linux-amd64 \
            -O ftl-resolve && chmod +x ftl-resolve && sudo mv ftl-resolve /usr/local/bin/
      
      - name: Login to registry
        run: echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u ${{ github.actor }} --password-stdin
      
      - name: Resolve and Transpile
        run: ftl-resolve spin ftl.toml -o spin.toml
      
      - name: Deploy with Spin
        run: spin deploy -f spin.toml
```

## Performance

- Minimal startup overhead
- Fresh pulls ensure latest components
- Components downloaded to `.ftl/wasm/` directory
- No caching - ensures consistency across deployments

## Support

For issues or questions:
- GitHub Issues: https://github.com/fastertools/ftl-cli/issues
- Documentation: https://docs.ftl.tools