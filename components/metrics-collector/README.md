# TEA (Telemetry & Event Recorder)

TEA is a high-performance telemetry and event recording service designed to collect, store, and analyze metrics and audit trails from MCP (Model Context Protocol) gateway operations. Built as a WebAssembly component, TEA provides real-time insights into system usage, performance, and billing data all while providing inherent isolation from user code.

## Overview

TEA serves as the central telemetry hub for the FTL ecosystem, capturing detailed metrics from MCP gateway middleware interactions. It handles authentication context, tracks resource usage, and provides comprehensive audit trails for compliance, billing, and monitoring purposes.

### Key Features

- **Real-time Telemetry Collection**: Captures metrics from MCP gateway middleware with minimal latency
- **Authentication Context Handling**: Processes OIDC/JWT tokens to extract tenant and user information
- **Audit Trail Support**: Detailed logging for compliance and security requirements
- **Billing Integration**: Structured data collection for usage-based billing systems

## Architecture

TEA follows a modular architecture designed for scalability and maintainability.

### Data Flow

1. **Collection**: MCP gateway middleware intercepts requests/responses
2. **Context Extraction**: Authentication data (JWT/OIDC) is parsed for tenant/user info
3. **Event Creation**: Structured telemetry events are generated
4. **Transmission**: Events are sent to TEA service via HTTP
5. **Storage**: Events are persisted using configured storage backend

## Configuration

TEA is configured through the MCP Gateway configuration system.

### MCP Gateway Integration

Configure TEA middleware is done with the following settings:

```toml
[mcp_gateway]
tea_enabled = true
tea_endpoint = "http://localhost:3001"
tea_auth_required = true
```

### Spin Configuration

In `spin.toml`, the TEA service is defined as:

```toml
[[trigger.http]]
component = "tea"
route = "/tea/..."

[component.tea]
source = "components/metrics-collector"
allowed_outbound_hosts = ["*"]
```

### Environment Variables

- `TEA_STORAGE_BACKEND`: Storage backend type (default: "memory")
- `TEA_MAX_EVENTS`: Maximum events to store in memory (default: 10000)
- `TEA_RETENTION_HOURS`: Event retention period (default: 24)
- `TEA_AUTH_VALIDATION`: Enable authentication validation (default: true)

## API Endpoints

TEA provides several HTTP endpoints for interacting with telemetry data:

### Event Ingestion

#### POST /events
Submit telemetry events to the system.

**Request Body:**
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "event_type": "mcp_request",
  "tenant_id": "tenant-123",
  "user_id": "user-456",
  "metadata": {
    "method": "tools/list",
    "duration_ms": 45,
    "status_code": 200,
    "request_size": 1024,
    "response_size": 2048
  }
}
```

**Response:**
```json
{
  "event_id": "evt_789",
  "status": "accepted",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Data Querying

#### GET /events
Query stored telemetry events with optional filtering.

**Query Parameters:**
- `tenant_id`: Filter by tenant ID
- `user_id`: Filter by user ID
- `event_type`: Filter by event type
- `start_time`: Start of time range (ISO 8601)
- `end_time`: End of time range (ISO 8601)
- `limit`: Maximum number of events to return (default: 100)

**Response:**
```json
{
  "events": [
    {
      "event_id": "evt_789",
      "timestamp": "2024-01-15T10:30:00Z",
      "event_type": "mcp_request",
      "tenant_id": "tenant-123",
      "user_id": "user-456",
      "metadata": {
        "method": "tools/list",
        "duration_ms": 45,
        "status_code": 200
      }
    }
  ],
  "total_count": 1,
  "has_more": false
}
```

#### GET /metrics
Aggregate metrics and statistics.

**Query Parameters:**
- `tenant_id`: Filter by tenant ID
- `metric_type`: Type of metric (requests, errors, latency, etc.)
- `granularity`: Time granularity (hour, day, week)
- `start_time`: Start of time range
- `end_time`: End of time range

**Response:**
```json
{
  "metrics": [
    {
      "timestamp": "2024-01-15T10:00:00Z",
      "total_requests": 1250,
      "error_rate": 0.02,
      "avg_latency_ms": 87,
      "total_data_mb": 15.6
    }
  ],
  "granularity": "hour"
}
```

### Health and Status

#### GET /health
Service health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "events_stored": 5432,
  "storage_backend": "memory"
}
```

## Authentication Context Handling

TEA automatically extracts and processes authentication information from MCP gateway requests:

### JWT Token Processing

```rust
// Example of authentication context extraction
pub struct AuthContext {
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub organization: Option<String>,
    pub scopes: Vec<String>,
    pub issued_at: Option<i64>,
    pub expires_at: Option<i64>,
}
```

### OIDC Integration

TEA supports standard OIDC claims:
- `sub`: Subject (user ID)
- `tenant`: Tenant identifier
- `org`: Organization
- `scope`: Granted permissions
- `iat`: Issued at timestamp
- `exp`: Expiration timestamp

## Usage Examples

### Basic Event Submission

```bash
# Submit a telemetry event
curl -X POST http://localhost:3001/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "mcp_request",
    "tenant_id": "acme-corp",
    "user_id": "john.doe",
    "metadata": {
      "method": "resources/list",
      "duration_ms": 120,
      "status_code": 200
    }
  }'
```

### Query Events for Billing

```bash
# Get all events for a tenant in the last 24 hours
curl "http://localhost:3001/events?tenant_id=acme-corp&start_time=2024-01-14T10:00:00Z&end_time=2024-01-15T10:00:00Z"
```

### Generate Usage Report

```bash
# Get aggregated metrics for billing
curl "http://localhost:3001/metrics?tenant_id=acme-corp&metric_type=requests&granularity=day&start_time=2024-01-01T00:00:00Z"
```

## Data Models

### TelemetryEvent

The core data structure for all telemetry events:

```rust
pub struct TelemetryEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: serde_json::Value,
    pub tags: Vec<String>,
}
```

### EventMetadata

Common metadata fields:

```rust
pub struct EventMetadata {
    pub method: Option<String>,
    pub duration_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub request_size: Option<u64>,
    pub response_size: Option<u64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}
```

## Development Setup

### Prerequisites

- Rust 1.70+
- Spin CLI 2.0+
- WebAssembly target: `rustup target add wasm32-wasi`

### Building

```bash
# Build the TEA component
cd components/metrics-collector
cargo build --target wasm32-wasi --release

# Build entire project including TEA
spin build
```

### Running Locally

```bash
# Start the Spin application (includes TEA service)
spin up --listen 127.0.0.1:3000

# TEA service will be available at:
# http://localhost:3001/
```

### Testing

```bash
# Run unit tests
cd components/metrics-collector
cargo test

# Test with sample data
curl -X POST http://localhost:3001/events \
  -H "Content-Type: application/json" \
  -d @test-event.json
```

### Development Workflow

1. **Make Changes**: Edit source files in `src/`
2. **Build**: Run `cargo build --target wasm32-wasi`
3. **Test**: Use `cargo test` for unit tests
4. **Integration Test**: Use `spin up` and test HTTP endpoints
5. **Deploy**: Build and deploy with `spin deploy`

## Storage Backends

TEA supports multiple storage backends:

### In-Memory Storage (Default)
- **Use Case**: Development, testing, short-term metrics
- **Retention**: Configurable, default 24 hours
- **Performance**: High throughput, low latency
- **Persistence**: None (data lost on restart)

### File-Based Storage
- **Use Case**: Single-node deployments, local development
- **Retention**: Configurable
- **Performance**: Moderate throughput
- **Persistence**: Local filesystem

### Database Storage (Future)
- **Use Case**: Production deployments, high availability
- **Retention**: Configurable with automatic cleanup
- **Performance**: Scalable
- **Persistence**: Distributed storage

## Monitoring and Observability

TEA provides several mechanisms for monitoring its own health and performance:

### Metrics Exposure

- Event ingestion rate
- Storage utilization
- Query performance
- Error rates
- Authentication success/failure rates

### Health Checks

- Storage backend connectivity
- Memory usage
- Event processing latency
- Configuration validation

## Security Considerations

### Authentication

- JWT token validation
- OIDC claim verification
- Tenant isolation
- User permission checks

### Data Privacy

- PII handling policies
- Data retention compliance
- Secure transmission (HTTPS)
- Access logging

### Audit Trails

- Complete request/response logging
- Authentication events
- Configuration changes
- Data access patterns

## Performance Characteristics

### Throughput

- **Event Ingestion**: 1000+ events/second (single instance)
- **Query Performance**: Sub-100ms for typical queries
- **Memory Usage**: ~50MB base + storage overhead
- **Storage Efficiency**: JSON compression, optional sampling

### Scalability

- Horizontal scaling via multiple Spin instances
- Load balancing support
- Partitioned storage by tenant
- Asynchronous processing pipeline

## Troubleshooting

### Common Issues

#### High Memory Usage
```bash
# Check current storage usage
curl http://localhost:3001/health

# Reduce retention period
export TEA_RETENTION_HOURS=6
```

#### Authentication Failures
```bash
# Verify JWT token structure
curl -H "Authorization: Bearer $TOKEN" http://localhost:3001/events

# Check authentication logs
spin logs --component tea
```

#### Storage Errors
```bash
# Verify storage backend configuration
spin config get tea

# Check filesystem permissions (file storage)
ls -la /path/to/storage/directory
```

### Debug Logging

Enable debug logging for detailed troubleshooting:

```bash
export RUST_LOG=tea=debug,metrics_collector=debug
spin up
```

## Contributing

### Code Organization

- `src/lib.rs`: Main service entry point
- `src/routes.rs`: HTTP endpoint handlers
- `src/models.rs`: Data models and serialization
- `src/storage.rs`: Storage backend implementations
- `src/auth.rs`: Authentication context handling

### Testing Guidelines

- Unit tests for all data models
- Integration tests for API endpoints
- Performance tests for high-load scenarios
- Security tests for authentication flows

### Documentation

- Update README for new features
- Document API changes in OpenAPI spec
- Include usage examples for new endpoints
- Update configuration documentation

## License

This project is part of the FTL CLI ecosystem and follows the same licensing terms.

## Changelog

### v0.1.0 (Current)
- Initial TEA implementation
- Basic telemetry event collection
- In-memory storage backend
- RESTful API endpoints
- JWT/OIDC authentication context
- MCP gateway middleware integration
- Health check and metrics endpoints

### Planned Features
- Database storage backends
- Event streaming capabilities
- Advanced analytics and reporting
- Grafana dashboard integration
- Alerting and notification system
- Data export capabilities