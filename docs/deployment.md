# Deployment Guide

This guide covers deploying FTL projects to various environments.

## Deployment Options

FTL projects can be deployed anywhere Spin runs:

1. **Fermyon Cloud** - Managed Spin platform
2. **Kubernetes** - Using Spin Operator
3. **Self-hosted** - On your own infrastructure
4. **Edge platforms** - Cloudflare, Fastly, etc.

## Local Development

### Development Server

```bash
# Start with auto-rebuild
ftl watch

# Start on specific port
ftl up --port 8080

# Build before running
ftl up --build
```

### Production Mode

```bash
# Build with optimizations
ftl build --release

# Run in production mode
ftl up --port 80
```

## Fermyon Cloud Deployment

### Prerequisites

1. Install Spin CLI:
   ```bash
   curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
   ```

2. Login to Fermyon Cloud:
   ```bash
   spin login
   ```

### Deploy

```bash
# From project root (with spin.toml)
spin deploy

# Custom app name
spin deploy --app-name my-mcp-server

# Deploy to specific environment
spin deploy --environment production
```

### Environment Variables

Set environment variables:

```bash
spin cloud variables set API_KEY="your-key"
spin cloud variables set DATABASE_URL="postgres://..."
```

Or use `.env` file:

```bash
spin deploy --variables-file .env.production
```

### Custom Domains

```bash
# Add custom domain
spin cloud domain add api.example.com

# List domains
spin cloud domain list
```

## Kubernetes Deployment

### Using Spin Operator

1. Install Spin Operator:
   ```bash
   kubectl apply -f https://github.com/fermyon/spin-operator/releases/download/v0.1.0/spin-operator.yaml
   ```

2. Create SpinApp resource:
   ```yaml
   # spinapp.yaml
   apiVersion: core.spinoperator.dev/v1alpha1
   kind: SpinApp
   metadata:
     name: my-mcp-server
   spec:
     image: "ghcr.io/username/my-mcp-server:latest"
     replicas: 3
     variables:
       - name: API_KEY
         valueFrom:
           secretKeyRef:
             name: api-secrets
             key: api-key
   ```

3. Deploy:
   ```bash
   kubectl apply -f spinapp.yaml
   ```

### Using Docker

Build Docker image with Spin:

```dockerfile
# Dockerfile
FROM scratch
COPY spin.toml .
COPY weather-tool/handler/target/wasm32-wasip1/release/handler.wasm ./weather-tool/handler.wasm
COPY github-tool/handler/target/wasm32-wasip1/release/handler.wasm ./github-tool/handler.wasm
```

```bash
# Build and push
docker build -t myregistry/my-mcp-server:latest .
docker push myregistry/my-mcp-server:latest
```

## Self-Hosted Deployment

### Systemd Service

Create service file:

```ini
# /etc/systemd/system/mcp-server.service
[Unit]
Description=MCP Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/mcp-server
ExecStart=/usr/local/bin/spin up --port 3000
Restart=always
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable mcp-server
sudo systemctl start mcp-server
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  mcp-server:
    image: ghcr.io/fermyon/spin:latest
    command: up --from ghcr.io/username/my-mcp-server:latest
    ports:
      - "3000:3000"
    environment:
      - API_KEY=${API_KEY}
    restart: unless-stopped
```

### Reverse Proxy

Nginx configuration:

```nginx
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Environment Configuration

### Using spin.toml

```toml
# spin.toml
[variables]
api_endpoint = { default = "https://api.example.com" }
log_level = { default = "info", secret = false }
api_key = { required = true, secret = true }

[[component]]
id = "weather"
environment = {
  API_ENDPOINT = "{{ api_endpoint }}",
  LOG_LEVEL = "{{ log_level }}",
  API_KEY = "{{ api_key }}"
}
```

### Runtime Variables

Set at deployment:

```bash
# Fermyon Cloud
spin deploy --variable api_key="secret-key"

# Self-hosted
API_KEY="secret-key" spin up
```

## Monitoring & Logging

### Application Logs

```bash
# Fermyon Cloud
spin cloud logs --follow

# Self-hosted with systemd
journalctl -u mcp-server -f

# Docker
docker logs -f mcp-server
```

### Health Checks

Add health endpoint:

```typescript
// TypeScript component
export const tools = [
  createTool({
    name: '_health',
    description: 'Health check endpoint',
    inputSchema: { type: 'object' },
    execute: async () => {
      return JSON.stringify({
        status: 'healthy',
        timestamp: new Date().toISOString()
      });
    }
  })
];
```

### Metrics

Use OpenTelemetry:

```rust
// Rust component
use opentelemetry::{global, metrics::*};

let meter = global::meter("mcp-component");
let counter = meter
    .u64_counter("requests_total")
    .with_description("Total requests")
    .init();

counter.add(1, &[KeyValue::new("method", "tool_call")]);
```

## Security Best Practices

### 1. HTTPS/TLS

Always use HTTPS in production:

```nginx
server {
    listen 443 ssl http2;
    ssl_certificate /etc/ssl/certs/cert.pem;
    ssl_certificate_key /etc/ssl/private/key.pem;
    # ... rest of config
}
```

### 2. Authentication

Implement authentication for MCP endpoints:

```typescript
execute: async (args, context) => {
  const authHeader = context.headers['authorization'];
  if (!isValidAuth(authHeader)) {
    throw new Error('Unauthorized');
  }
  // ... tool logic
}
```

### 3. Rate Limiting

Protect against abuse:

```typescript
const rateLimiter = new Map();

execute: async (args, context) => {
  const clientId = context.clientId;
  if (isRateLimited(clientId)) {
    throw new Error('Rate limit exceeded');
  }
  // ... tool logic
}
```

### 4. Secrets Management

Never hardcode secrets:

```bash
# Use environment variables
export API_KEY="secret-key"

# Use secret management services
vault kv get secret/mcp/api-key

# Use Kubernetes secrets
kubectl create secret generic api-secrets --from-literal=api-key=secret-key
```

## Scaling

### Horizontal Scaling

```yaml
# Kubernetes
spec:
  replicas: 5
  
# Docker Swarm
docker service scale mcp-server=5

# Fermyon Cloud
spin cloud app scale 5
```

### Load Balancing

```nginx
upstream mcp_servers {
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    server 127.0.0.1:3003;
}

server {
    location / {
        proxy_pass http://mcp_servers;
    }
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install FTL
        run: cargo install ftl-cli
      
      - name: Build
        run: ftl build --release
      
      - name: Deploy to Fermyon Cloud
        env:
          SPIN_AUTH_TOKEN: ${{ secrets.SPIN_AUTH_TOKEN }}
        run: spin deploy --app-name production-mcp
```

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - cargo install ftl-cli
    - ftl build --release
    - spin deploy
  only:
    - main
```

## Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   # Find process using port
   lsof -i :3000
   # Kill process
   kill -9 <PID>
   ```

2. **Component crashes**
   - Check logs for errors
   - Verify environment variables
   - Test locally first

3. **Performance issues**
   - Monitor resource usage
   - Optimize component code
   - Scale horizontally

### Debug Mode

Enable debug logging:

```bash
# Local
RUST_LOG=debug ftl up

# Fermyon Cloud
spin cloud variables set RUST_LOG=debug
```

## Next Steps

- [Monitoring Guide](./monitoring.md) - Set up observability
- [Security Guide](./security.md) - Secure your deployment
- [Performance Guide](./performance.md) - Optimize for production