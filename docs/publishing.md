# Publishing Components

This guide covers how to publish and share your FTL components via OCI registries.

## Prerequisites

Before publishing, you'll need:
1. An OCI registry account (GitHub, Docker Hub, etc.)
2. Authentication configured for your registry
3. `wkg` tool installed (optional but recommended)

## Registry Setup

### GitHub Container Registry (ghcr.io)

1. Create a Personal Access Token:
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Create a token with `write:packages` permission

2. Login to the registry:
   ```bash
   echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
   ```

### Docker Hub

1. Create an account at [hub.docker.com](https://hub.docker.com)

2. Login to the registry:
   ```bash
   docker login
   ```

## Publishing Workflow

### 1. Build Your Component

Ensure your component builds successfully:

```bash
cd my-component
ftl build --release
```

### 2. Update Metadata

Edit your component's `ftl.toml`:

```toml
name = "weather-tool"
version = "1.0.0"
description = "Real-time weather data for AI agents"
authors = ["Your Name <you@example.com>"]
license = "MIT"
repository = "https://github.com/username/weather-tool"
keywords = ["weather", "mcp", "tool"]
```

### 3. Test Locally

Run final tests before publishing:

```bash
ftl test
ftl up --port 3000
```

### 4. Publish to Registry

#### Default Registry (ghcr.io)

```bash
# Publish with a version tag
ftl publish --tag v1.0.0

# This publishes to: ghcr.io/YOUR_USERNAME/weather-tool:v1.0.0
```

#### Custom Registry

```bash
# Publish to Docker Hub
ftl publish --registry docker.io --tag latest

# Publish to a private registry
ftl publish --registry registry.company.com --tag v1.0.0
```

## Version Management

### Semantic Versioning

Follow semantic versioning for your components:
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes

### Version Tags

```bash
# Publish specific version
ftl publish --tag v1.2.3

# Publish latest tag
ftl publish --tag latest

# Publish with multiple tags
ftl publish --tag v1.2.3
ftl publish --tag v1.2
ftl publish --tag v1
ftl publish --tag latest
```

## Using Published Components

### In FTL Projects

Reference published components in your project:

```bash
# Add a published component to your project
ftl add weather --from ghcr.io/username/weather-tool:v1.0.0
```

### Direct Usage with Spin

```toml
# spin.toml
[[component]]
id = "weather"
source = { registry = "ghcr.io/username/weather-tool:v1.0.0" }
route = "/weather/..."
```

## Component Discovery

### Public Registries

Browse public components:
- GitHub: `https://github.com/orgs/ORG/packages`
- Docker Hub: `https://hub.docker.com/search`

### Component Metadata

Well-documented components include:
- Clear descriptions
- Usage examples
- API documentation
- License information

## Best Practices

### 1. Documentation

Include comprehensive documentation:

```markdown
# Weather Tool

Real-time weather data for AI agents.

## Installation

```bash
ftl add weather --from ghcr.io/username/weather-tool:latest
```

## Tools

- `get_weather`: Get current weather for a location
- `get_forecast`: Get weather forecast

## Usage

```typescript
const result = await callTool('get_weather', {
  location: 'San Francisco',
  units: 'fahrenheit'
});
```
```

### 2. Changelog

Maintain a CHANGELOG.md:

```markdown
# Changelog

## [1.2.0] - 2024-01-15
### Added
- Support for weather alerts
- Metric units option

### Fixed
- Timezone handling for forecasts

## [1.1.0] - 2024-01-01
### Added
- 7-day forecast capability
```

### 3. Testing

Include example tests:

```typescript
// examples/test-weather.ts
import { Client } from '@modelcontextprotocol/sdk';

const client = new Client({
  url: 'http://localhost:3000/weather/mcp'
});

const weather = await client.callTool('get_weather', {
  location: 'London'
});
console.log(weather);
```

### 4. Security

- Never include secrets in components
- Use environment variables for configuration
- Document required permissions
- Keep dependencies updated

### 5. Component Size

Optimize component size:
- Minimize dependencies
- Use tree-shaking for JavaScript
- Enable release optimizations
- Consider splitting large components

## Advanced Publishing

### Multi-Architecture Support

Build for multiple architectures:

```bash
# Build for multiple targets
ftl build --target wasm32-wasip1
ftl build --target wasm32-wasip2
```

### Automated Publishing

GitHub Actions example:

```yaml
name: Publish Component

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install FTL
        run: cargo install ftl-cli
      
      - name: Build component
        run: ftl build --release
      
      - name: Login to ghcr.io
        run: echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u ${{ github.actor }} --password-stdin
      
      - name: Publish
        run: ftl publish --tag ${GITHUB_REF#refs/tags/}
```

### Component Signing

Sign your components for security:

```bash
# Sign with cosign
cosign sign ghcr.io/username/weather-tool:v1.0.0

# Verify signature
cosign verify ghcr.io/username/weather-tool:v1.0.0
```

## Registry Management

### List Published Versions

```bash
# Using wkg
wkg list ghcr.io/username/weather-tool

# Using docker
docker run --rm gcr.io/go-containerregistry/crane ls ghcr.io/username/weather-tool
```

### Delete Versions

```bash
# Delete specific version
wkg delete ghcr.io/username/weather-tool:v0.1.0

# Delete using docker
docker run --rm gcr.io/go-containerregistry/crane delete ghcr.io/username/weather-tool:v0.1.0
```

## Troubleshooting

### Authentication Issues

```bash
# Check current auth
docker config get-credential-helpers

# Re-authenticate
docker logout ghcr.io
docker login ghcr.io
```

### Publishing Failures

Common issues and solutions:

1. **Permission denied**
   - Check registry authentication
   - Verify token permissions

2. **Version already exists**
   - Use a different version tag
   - Delete existing version if needed

3. **Size limits**
   - Optimize component size
   - Check registry limits

### Registry Debugging

```bash
# Verbose output
ftl publish --tag v1.0.0 --verbose

# Check component size
du -h my-component/handler/target/wasm32-wasip1/release/*.wasm
```

## Next Steps

- [Component Development](./components.md) - Build better components
- [Deployment Guide](./deployment.md) - Deploy published components
- [CLI Reference](./cli-reference.md) - Publishing command options