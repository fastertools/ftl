# Testing Your Tools

**Problem**: How to write comprehensive tests for your FTL tools to ensure they work correctly and handle edge cases.

**Solution**: Use language-specific testing frameworks with FTL-aware patterns for unit tests, integration tests, and end-to-end testing.

## Overview

Testing FTL tools requires a multi-layered approach:

1. **Unit Tests**: Test individual tool functions in isolation
2. **Integration Tests**: Test tools within the FTL runtime environment  
3. **MCP Protocol Tests**: Verify MCP compliance and tool schemas
4. **End-to-End Tests**: Test complete client-to-tool workflows

## Language-Specific Testing Strategies

### Rust Testing

Rust provides excellent built-in testing support that works well with FTL tools.

#### Unit Tests

```rust
// components/my-tool/src/lib.rs
use ftl_sdk::prelude::*;
use serde_json::json;

#[tool]
pub fn calculate_sum(numbers: Vec<f64>) -> ToolResponse {
    if numbers.is_empty() {
        return ToolResponse::error("Numbers list cannot be empty");
    }
    
    let sum: f64 = numbers.iter().sum();
    ToolResponse::ok(&format!("{}", sum))
}

#[tool]
pub fn validate_email(email: String) -> ToolResponse {
    if email.contains('@') && email.contains('.') {
        ToolResponse::ok("Valid email")
    } else {
        ToolResponse::error("Invalid email format")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sum_success() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0];
        let result = calculate_sum(numbers);
        
        assert!(result.is_ok());
        assert_eq!(result.content, "10");
    }

    #[test]
    fn test_calculate_sum_empty_list() {
        let numbers = vec![];
        let result = calculate_sum(numbers);
        
        assert!(result.is_error());
        assert_eq!(result.content, "Numbers list cannot be empty");
    }

    #[test]
    fn test_validate_email_valid() {
        let result = validate_email("test@example.com".to_string());
        assert!(result.is_ok());
        assert_eq!(result.content, "Valid email");
    }

    #[test]
    fn test_validate_email_invalid() {
        let result = validate_email("invalid-email".to_string());
        assert!(result.is_error());
        assert_eq!(result.content, "Invalid email format");
    }

    #[test]
    fn test_email_validation_edge_cases() {
        let test_cases = vec![
            ("", false),
            ("@", false),
            ("test@", false),
            ("@example.com", false),
            ("test@example.com", true),
            ("user.name+tag@example.co.uk", true),
        ];

        for (email, should_be_valid) in test_cases {
            let result = validate_email(email.to_string());
            if should_be_valid {
                assert!(result.is_ok(), "Expected {} to be valid", email);
            } else {
                assert!(result.is_error(), "Expected {} to be invalid", email);
            }
        }
    }
}
```

#### Async Tool Testing

```rust
use tokio_test;

#[tool]
pub async fn fetch_user_data(user_id: u32) -> ToolResponse {
    // Simulate async operation
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    if user_id == 0 {
        return ToolResponse::error("Invalid user ID");
    }
    
    let user_data = format!("{{\"id\": {}, \"name\": \"User {}\"}}", user_id, user_id);
    ToolResponse::ok(&user_data)
}

#[cfg(test)]
mod async_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_user_data_success() {
        let result = fetch_user_data(123).await;
        assert!(result.is_ok());
        assert!(result.content.contains("\"id\": 123"));
    }

    #[tokio::test]
    async fn test_fetch_user_data_invalid_id() {
        let result = fetch_user_data(0).await;
        assert!(result.is_error());
        assert_eq!(result.content, "Invalid user ID");
    }
}
```

#### Running Rust Tests

```bash
cd components/my-tool
cargo test                    # Run all tests
cargo test test_calculate     # Run specific test
cargo test --release         # Test optimized build
cargo test -- --nocapture    # Show print statements
```

### Python Testing

Python testing with pytest provides powerful testing capabilities.

#### Setup Testing Environment

```toml
# components/my-tool/pyproject.toml
[project]
dependencies = [
    "ftl-sdk",
    "httpx",
    "pydantic"
]

[project.optional-dependencies]
test = [
    "pytest",
    "pytest-asyncio",
    "pytest-mock",
    "httpx",
    "responses"  # For mocking HTTP requests
]
```

#### Unit Tests

```python
# components/my-tool/src/__init__.py
from ftl_sdk import tool, ToolResponse
import httpx
from typing import List

@tool
def calculate_average(numbers: List[float]) -> ToolResponse:
    """Calculate the average of a list of numbers."""
    if not numbers:
        return ToolResponse.error("Numbers list cannot be empty")
    
    if not all(isinstance(n, (int, float)) for n in numbers):
        return ToolResponse.error("All items must be numbers")
    
    average = sum(numbers) / len(numbers)
    return ToolResponse.ok(f"{average}")

@tool
async def fetch_weather(city: str) -> ToolResponse:
    """Fetch weather data for a city."""
    if not city or not city.strip():
        return ToolResponse.error("City name cannot be empty")
    
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            response = await client.get(
                f"https://api.weather.com/v1/current?city={city}",
                headers={"API-Key": "demo-key"}
            )
            response.raise_for_status()
            
            data = response.json()
            temperature = data.get("temperature", "Unknown")
            return ToolResponse.ok(f"Temperature in {city}: {temperature}°C")
            
    except httpx.HTTPError as e:
        return ToolResponse.error(f"Weather API error: {str(e)}")
    except Exception as e:
        return ToolResponse.error(f"Unexpected error: {str(e)}")
```

```python
# components/my-tool/tests/test_tools.py
import pytest
import httpx
import responses
from unittest.mock import patch
from src import calculate_average, fetch_weather

class TestCalculateAverage:
    def test_calculate_average_success(self):
        result = calculate_average([1.0, 2.0, 3.0, 4.0])
        assert result.is_ok()
        assert result.content == "2.5"

    def test_calculate_average_empty_list(self):
        result = calculate_average([])
        assert result.is_error()
        assert "empty" in result.content.lower()

    def test_calculate_average_invalid_types(self):
        result = calculate_average([1, 2, "three", 4])
        assert result.is_error()
        assert "numbers" in result.content.lower()

    @pytest.mark.parametrize("numbers,expected", [
        ([5.0], "5.0"),
        ([0.0, 0.0], "0.0"),
        ([-1.0, 1.0], "0.0"),
        ([1, 2, 3], "2.0"),
    ])
    def test_calculate_average_edge_cases(self, numbers, expected):
        result = calculate_average(numbers)
        assert result.is_ok()
        assert result.content == expected

class TestFetchWeather:
    @pytest.mark.asyncio
    async def test_fetch_weather_empty_city(self):
        result = await fetch_weather("")
        assert result.is_error()
        assert "empty" in result.content.lower()

    @pytest.mark.asyncio
    @responses.activate  # Mock HTTP requests
    async def test_fetch_weather_success(self):
        responses.add(
            responses.GET,
            "https://api.weather.com/v1/current",
            json={"temperature": 22.5, "humidity": 65},
            status=200
        )
        
        result = await fetch_weather("London")
        assert result.is_ok()
        assert "London" in result.content
        assert "22.5°C" in result.content

    @pytest.mark.asyncio
    @responses.activate
    async def test_fetch_weather_api_error(self):
        responses.add(
            responses.GET,
            "https://api.weather.com/v1/current",
            json={"error": "Invalid API key"},
            status=401
        )
        
        result = await fetch_weather("London")
        assert result.is_error()
        assert "Weather API error" in result.content

    @pytest.mark.asyncio
    async def test_fetch_weather_timeout(self):
        with patch('httpx.AsyncClient.get') as mock_get:
            mock_get.side_effect = httpx.TimeoutException("Request timeout")
            
            result = await fetch_weather("London")
            assert result.is_error()
            assert "timeout" in result.content.lower()
```

#### Running Python Tests

```bash
cd components/my-tool
pip install -e ".[test]"       # Install with test dependencies
pytest                         # Run all tests
pytest tests/test_tools.py     # Run specific file
pytest -v                      # Verbose output
pytest --cov=src               # Test coverage
pytest -k "test_weather"       # Run tests matching pattern
```

### Go Testing

Go's built-in testing framework works well for FTL tools.

#### Unit Tests

```go
// components/my-tool/main.go
//go:build wasip1

package main

import (
    "context"
    "encoding/json"
    "fmt"
    "strconv"
    "strings"

    "go.bytecodealliance.org/cm"
)

type MathToolImpl struct{}

func (m *MathToolImpl) Add(ctx context.Context, args string) cm.Result[string, string] {
    var req struct {
        A float64 `json:"a"`
        B float64 `json:"b"`
    }
    
    if err := json.Unmarshal([]byte(args), &req); err != nil {
        return cm.Err[string]("Invalid arguments: " + err.Error())
    }
    
    result := req.A + req.B
    return cm.OK[string](fmt.Sprintf("%.2f", result))
}

func (m *MathToolImpl) ParseNumbers(ctx context.Context, input string) cm.Result[string, string] {
    if strings.TrimSpace(input) == "" {
        return cm.Err[string]("Input cannot be empty")
    }
    
    parts := strings.Split(input, ",")
    numbers := make([]float64, 0, len(parts))
    
    for _, part := range parts {
        trimmed := strings.TrimSpace(part)
        if num, err := strconv.ParseFloat(trimmed, 64); err == nil {
            numbers = append(numbers, num)
        }
    }
    
    if len(numbers) == 0 {
        return cm.Err[string]("No valid numbers found")
    }
    
    result, _ := json.Marshal(numbers)
    return cm.OK[string](string(result))
}
```

```go
// components/my-tool/main_test.go
package main

import (
    "context"
    "encoding/json"
    "testing"

    "go.bytecodealliance.org/cm"
)

func TestMathToolImpl_Add(t *testing.T) {
    tool := &MathToolImpl{}
    ctx := context.Background()

    tests := []struct {
        name     string
        args     string
        want     string
        wantErr  bool
    }{
        {
            name: "successful addition",
            args: `{"a": 2.5, "b": 3.7}`,
            want: "6.20",
            wantErr: false,
        },
        {
            name: "zero values",
            args: `{"a": 0, "b": 0}`,
            want: "0.00",
            wantErr: false,
        },
        {
            name: "negative numbers",
            args: `{"a": -5.5, "b": 2.3}`,
            want: "-3.20",
            wantErr: false,
        },
        {
            name: "invalid JSON",
            args: `{"a": 2.5, "b":}`,
            want: "",
            wantErr: true,
        },
        {
            name: "missing field",
            args: `{"a": 2.5}`,
            want: "",
            wantErr: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            result := tool.Add(ctx, tt.args)
            
            if tt.wantErr {
                if result.IsOK() {
                    t.Errorf("Add() expected error but got success: %v", result.Unwrap())
                }
            } else {
                if result.IsErr() {
                    t.Errorf("Add() unexpected error: %v", result.UnwrapErr())
                    return
                }
                
                got := result.Unwrap()
                if got != tt.want {
                    t.Errorf("Add() = %v, want %v", got, tt.want)
                }
            }
        })
    }
}

func TestMathToolImpl_ParseNumbers(t *testing.T) {
    tool := &MathToolImpl{}
    ctx := context.Background()

    tests := []struct {
        name        string
        input       string
        wantNumbers []float64
        wantErr     bool
    }{
        {
            name: "comma-separated numbers",
            input: "1.5, 2.7, 3.9",
            wantNumbers: []float64{1.5, 2.7, 3.9},
            wantErr: false,
        },
        {
            name: "mixed valid and invalid",
            input: "1, abc, 2.5, def, 3",
            wantNumbers: []float64{1, 2.5, 3},
            wantErr: false,
        },
        {
            name: "empty input",
            input: "",
            wantNumbers: nil,
            wantErr: true,
        },
        {
            name: "no valid numbers",
            input: "abc, def, ghi",
            wantNumbers: nil,
            wantErr: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            result := tool.ParseNumbers(ctx, tt.input)
            
            if tt.wantErr {
                if result.IsOK() {
                    t.Errorf("ParseNumbers() expected error but got success")
                }
                return
            }
            
            if result.IsErr() {
                t.Errorf("ParseNumbers() unexpected error: %v", result.UnwrapErr())
                return
            }
            
            var got []float64
            if err := json.Unmarshal([]byte(result.Unwrap()), &got); err != nil {
                t.Errorf("Failed to unmarshal result: %v", err)
                return
            }
            
            if len(got) != len(tt.wantNumbers) {
                t.Errorf("ParseNumbers() length = %v, want %v", len(got), len(tt.wantNumbers))
                return
            }
            
            for i, num := range got {
                if num != tt.wantNumbers[i] {
                    t.Errorf("ParseNumbers()[%d] = %v, want %v", i, num, tt.wantNumbers[i])
                }
            }
        })
    }
}

// Benchmark tests
func BenchmarkAdd(b *testing.B) {
    tool := &MathToolImpl{}
    ctx := context.Background()
    args := `{"a": 123.456, "b": 789.012}`
    
    b.ResetTimer()
    for i := 0; i < b.N; i++ {
        tool.Add(ctx, args)
    }
}
```

#### Running Go Tests

```bash
cd components/my-tool
go test                        # Run all tests  
go test -v                     # Verbose output
go test -run TestAdd           # Run specific test
go test -bench=.               # Run benchmarks
go test -cover                 # Test coverage
go test -race                  # Race condition detection
```

## Integration Testing

### Testing with FTL Runtime

Create integration tests that run tools within the FTL environment:

```bash
# integration_test.sh
#!/bin/bash

set -e

echo "🔨 Building FTL project..."
ftl build

echo "🚀 Starting FTL server..."
ftl up &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo "🧪 Running integration tests..."

# Test tool list
echo "Testing tool list..."
RESPONSE=$(curl -s -X POST http://localhost:3000/tools/list \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}')

if ! echo "$RESPONSE" | grep -q '"result"'; then
    echo "❌ Tool list test failed"
    echo "Response: $RESPONSE"
    exit 1
fi

# Test tool call
echo "Testing tool call..."
RESPONSE=$(curl -s -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "my-tool/calculate_sum",
      "arguments": {"numbers": [1, 2, 3, 4]}
    }
  }')

if ! echo "$RESPONSE" | grep -q '"result"'; then
    echo "❌ Tool call test failed"
    echo "Response: $RESPONSE"
    exit 1
fi

echo "✅ All integration tests passed!"

# Cleanup
kill $SERVER_PID
```

### Schema Validation Testing

Test that your tools generate valid MCP schemas:

```python
# test_schemas.py
import json
import jsonschema
import requests
from pathlib import Path

def test_tool_schemas():
    """Test that generated tool schemas are valid."""
    
    # Start FTL server (or assume it's running)
    response = requests.post(
        "http://localhost:3000/tools/list",
        json={
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }
    )
    
    assert response.status_code == 200
    data = response.json()
    assert "result" in data
    
    tools = data["result"]["tools"]
    assert len(tools) > 0
    
    for tool in tools:
        # Validate tool structure
        assert "name" in tool
        assert "description" in tool
        assert "inputSchema" in tool
        
        # Validate JSON Schema
        schema = tool["inputSchema"]
        jsonschema.Draft7Validator.check_schema(schema)
        
        print(f"✅ Schema valid for tool: {tool['name']}")

if __name__ == "__main__":
    test_tool_schemas()
```

## End-to-End Testing

### MCP Client Testing

Test with a real MCP client:

```python
# e2e_test.py
import asyncio
import json
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

async def test_mcp_client():
    """Test FTL server with a real MCP client."""
    
    # Configure MCP client to connect to FTL server
    server_params = StdioServerParameters(
        command="curl",
        args=[
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "http://localhost:3000/tools/list"
        ]
    )
    
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            # Initialize session
            await session.initialize()
            
            # List available tools
            tools = await session.list_tools()
            print(f"Available tools: {[tool.name for tool in tools.tools]}")
            
            # Call a tool
            result = await session.call_tool(
                "my-tool/calculate_sum",
                {"numbers": [1, 2, 3, 4, 5]}
            )
            
            print(f"Tool result: {result.content}")
            assert len(result.content) > 0

# Run the test
asyncio.run(test_mcp_client())
```

### Load Testing

Test performance under load:

```bash
# load_test.sh
#!/bin/bash

# Start FTL server
ftl up &
SERVER_PID=$!
sleep 3

# Install Apache Bench if not available
# brew install httpie

# Load test tool calls
echo "🔥 Running load test..."
ab -n 1000 -c 10 \
  -T 'application/json' \
  -p <(echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"my-tool/calculate_sum","arguments":{"numbers":[1,2,3]}}}') \
  http://localhost:3000/tools/call

# Cleanup
kill $SERVER_PID
```

## Continuous Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Test FTL Tools

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install FTL CLI
      run: |
        curl -fsSL https://cli.ftlengine.dev/install.sh | bash
        echo "$HOME/.ftl/bin" >> $GITHUB_PATH
    
    - name: Install language runtimes
      run: |
        # Install Rust
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        rustup target add wasm32-wasip1
        
        # Install Python
        sudo apt-get update
        sudo apt-get install -y python3 python3-pip
        
        # Install Go
        wget https://go.dev/dl/go1.21.0.linux-amd64.tar.gz
        sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
        echo "/usr/local/go/bin" >> $GITHUB_PATH
    
    - name: Build FTL project
      run: ftl build
    
    - name: Run unit tests
      run: |
        # Test Rust components
        for component in components/*/Cargo.toml; do
          dir=$(dirname "$component")
          echo "Testing $dir"
          cd "$dir"
          cargo test
          cd ../..
        done
        
        # Test Python components
        for component in components/*/pyproject.toml; do
          dir=$(dirname "$component")
          echo "Testing $dir"
          cd "$dir"
          pip install -e ".[test]"
          pytest
          cd ../..
        done
    
    - name: Run integration tests
      run: |
        ./scripts/integration_test.sh
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: test-results/
```

## Test Organization Best Practices

### Directory Structure
```
my-ftl-project/
├── components/
│   ├── tool1/
│   │   ├── src/
│   │   ├── tests/           # Unit tests
│   │   └── Cargo.toml
│   └── tool2/
│       ├── src/
│       ├── tests/           # Unit tests
│       └── pyproject.toml
├── tests/
│   ├── integration/         # Integration tests
│   ├── e2e/                # End-to-end tests
│   └── fixtures/           # Test data
└── scripts/
    ├── test.sh             # Test runner
    └── load_test.sh        # Performance tests
```

### Test Data Management

```python
# tests/fixtures/data.py
"""Test data fixtures for FTL tools."""

VALID_EMAILS = [
    "user@example.com",
    "test.email+tag@domain.co.uk",
    "valid.email@subdomain.example.com",
]

INVALID_EMAILS = [
    "",
    "@",
    "user@",
    "@domain.com",
    "not-an-email",
]

SAMPLE_NUMBERS = {
    "small": [1, 2, 3],
    "large": list(range(1000)),
    "floats": [1.5, 2.7, 3.9],
    "negative": [-1, -2, -3],
    "mixed": [-1.5, 0, 2.7, 100],
}

HTTP_RESPONSES = {
    "success": {"status": "ok", "data": {"result": "success"}},
    "error": {"status": "error", "message": "Something went wrong"},
    "empty": {},
}
```

### Mock External Dependencies

```python
# tests/mocks.py
"""Mock external services for testing."""

import responses
import httpx
from unittest.mock import AsyncMock

class MockWeatherAPI:
    """Mock weather API responses."""
    
    @staticmethod
    @responses.activate
    def setup_success(city="London", temp=22.5):
        responses.add(
            responses.GET,
            f"https://api.weather.com/v1/current",
            json={"temperature": temp, "city": city},
            status=200
        )
    
    @staticmethod
    @responses.activate 
    def setup_error(status_code=500):
        responses.add(
            responses.GET,
            "https://api.weather.com/v1/current",
            json={"error": "Internal server error"},
            status=status_code
        )

# Usage in tests
def test_weather_tool_success():
    MockWeatherAPI.setup_success("Paris", 25.0)
    result = fetch_weather("Paris")
    assert result.is_ok()
    assert "25.0°C" in result.content
```

## Troubleshooting Tests

### Common Issues

**WASM compilation errors in tests**:
- Tests run in native mode by default
- Use `#[cfg(not(target_arch = "wasm32"))]` for native-only test code
- Mock WASM-specific functionality for unit tests

**Async test failures**:
```rust
// Add to Cargo.toml for async testing
[dev-dependencies]
tokio-test = "0.4"

// Use tokio-test for async unit tests
#[tokio::test]
async fn my_async_test() {
    // Test async code
}
```

**HTTP mocking issues**:
```python
# Ensure responses mock is activated
@responses.activate
def test_with_http_mock():
    responses.add(responses.GET, "https://api.example.com", json={})
    # Your test code
```

**Integration test timeouts**:
```bash
# Increase server startup wait time
sleep 5  # Instead of sleep 3

# Add health check loop
while ! curl -f http://localhost:3000/_health; do
    sleep 1
done
```

## Next Steps

- **HTTP Requests**: Test external API calls in [Making HTTP Requests](./http-requests.md)
- **Authentication**: Test authenticated tools in [Handling Authentication](./authentication.md)
- **Advanced Patterns**: Explore testing strategies in [Examples](../../examples/)
- **SDK Reference**: Check testing utilities in [SDK Reference](../sdk-reference/)

Comprehensive testing ensures your FTL tools work correctly in all scenarios, from development through production deployment. The multi-layered testing approach catches issues early and gives confidence in your tool implementations.