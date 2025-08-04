# MCP Gateway Test Suite

This directory contains comprehensive tests for the FTL MCP Gateway component using the Spin Test framework.

## Overview

The test suite validates the MCP Gateway's compliance with the Model Context Protocol specification, including:

- Protocol implementation (initialization, capabilities)
- Tool discovery and routing
- Argument validation
- Error handling
- CORS support
- JSON-RPC compliance
- Integration scenarios

## Test Structure

```
tests/
├── src/
│   ├── lib.rs                    # Main test module and shared utilities
│   ├── test_helpers.rs           # Common test helper functions
│   ├── basic_test.rs             # Basic functionality tests
│   ├── protocol_tests.rs         # MCP protocol implementation tests
│   ├── routing_tests.rs          # Tool routing and discovery tests
│   ├── validation_tests.rs       # Argument validation tests
│   ├── error_handling_tests.rs   # Error scenarios and recovery
│   ├── tool_discovery_tests.rs   # Tool metadata and listing tests
│   ├── cors_tests.rs             # CORS header handling tests
│   ├── json_rpc_tests.rs         # JSON-RPC protocol compliance
│   ├── performance_tests.rs      # Performance and scalability tests
│   └── integration_tests.rs      # End-to-end integration scenarios
├── Cargo.toml                    # Test dependencies and configuration
├── spin-test.toml                # Spin test framework configuration
└── run_tests.sh                  # Test execution script
```

## Running Tests

### Prerequisites

- Rust toolchain with `wasm32-wasip1` target
- Spin CLI with spin-test plugin installed

### Run All Tests

```bash
./run_tests.sh
```

### Run Specific Test Module

```bash
spin-test --filter protocol_tests
```

### Run Single Test

```bash
spin-test --filter test_initialize_protocol_v1
```

## Test Categories

### Protocol Tests (`protocol_tests.rs`)
- MCP initialization handshake
- Protocol version negotiation
- Server capabilities declaration
- Notification handling

### Routing Tests (`routing_tests.rs`)
- Tool name resolution (snake_case to kebab-case conversion)
- Component discovery
- Parallel tool fetching
- Error handling for missing components

### Validation Tests (`validation_tests.rs`)
- JSON Schema validation for tool arguments
- Validation enable/disable behavior
- Complex nested schema validation
- Error message formatting

### Error Handling Tests (`error_handling_tests.rs`)
- Invalid JSON-RPC requests
- Method not found errors
- Tool execution failures
- Component communication errors
- Graceful degradation

### Tool Discovery Tests (`tool_discovery_tests.rs`)
- Empty tool lists
- Multiple component aggregation
- Metadata completeness
- Duplicate tool name handling

### CORS Tests (`cors_tests.rs`)
- Preflight OPTIONS requests
- CORS headers on all responses
- Method restrictions (POST/OPTIONS only)

### JSON-RPC Tests (`json_rpc_tests.rs`)
- Version validation
- ID type handling (number, string, null)
- Notification behavior
- Error response formatting

### Performance Tests (`performance_tests.rs`)
- Concurrent component queries
- Large tool list handling
- Complex schema validation performance

### Integration Tests (`integration_tests.rs`)
- Full MCP session flow
- Multiple tool invocations
- Error recovery scenarios
- Mixed component states

## Writing New Tests

### Test Helper Functions

Use the provided helpers in `test_helpers.rs`:

```rust
// Create JSON-RPC request
let request = create_json_rpc_request("method", params, id);

// Create MCP HTTP request
let http_request = create_mcp_request(json_rpc);

// Verify successful response
assert_json_rpc_success(&response_json, expected_id);

// Verify error response
assert_json_rpc_error(&response_json, error_code, expected_id);
```

### Mock Component Setup

Use `http_handler::add_request_handler` to mock tool components:

```rust
http_handler::add_request_handler(|request, _route_params| {
    if request.method() == http::types::Method::Get {
        // Return tool metadata
        let tools = vec![...];
        // ...
    } else if request.method() == http::types::Method::Post {
        // Handle tool execution
        // ...
    }
});
```

## Test Coverage Goals

The test suite provides comprehensive validation:

- Complete MCP specification compliance
- Protocol edge cases and error conditions
- Component integration scenarios
- Performance and scalability characteristics
- Cross-cutting concerns (CORS, validation, routing)

## Contributing

When adding new tests:

1. Follow existing naming conventions
2. Use descriptive test names
3. Include comments for complex scenarios
4. Ensure tests are deterministic
5. Mock external dependencies
6. Test both success and failure paths

## Troubleshooting

### Common Issues

1. **Tests fail to compile**: Ensure `wasm32-wasip1` target is installed (`rustup target add wasm32-wasip1`)
2. **Spin-test not found**: Install the plugin with `spin plugins install spin-test`
3. **Mock handlers not working**: Verify variable names match between test configuration and mock handlers
4. **Component communication failures**: Check that component names use proper kebab-case conversion

### Debug Output

Enable debug logging:

```bash
RUST_LOG=debug spin-test
```