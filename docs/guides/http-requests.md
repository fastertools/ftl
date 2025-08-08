# Making HTTP Requests from Tools

**Problem**: Your FTL tool needs to call external APIs, fetch data from web services, or send HTTP requests to third-party services.

**Solution**: Configure outbound network access and implement HTTP client code using your language's standard libraries.

## Overview

FTL tools run in WebAssembly sandboxes with restricted network access by default. To make HTTP requests, you need to:

1. **Whitelist external hosts** in your tool configuration
2. **Use appropriate HTTP client libraries** for your language
3. **Handle errors and timeouts** appropriately
4. **Manage authentication** for external services

## Step 1: Configure Outbound Network Access

First, whitelist the external hosts your tool needs to access in `ftl.toml`:

```toml
[tools.my-api-tool]
path = "components/my-api-tool"
allowed_outbound_hosts = [
    "https://api.openai.com",
    "https://httpbin.org",
    "https://api.github.com"
]
```

**Important**: 
- Include the full protocol (`https://`) and domain
- Wildcards are not supported - each host must be explicitly listed
- Changes require rebuilding: `ftl build`

## Step 2: Implement HTTP Requests by Language

### Rust

Use the `reqwest` crate for HTTP requests:

```toml
# components/my-tool/Cargo.toml
[dependencies]
ftl-sdk = "0.1.0"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt"] }
```

```rust
// components/my-tool/src/lib.rs
use ftl_sdk::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: String,
    pub data: serde_json::Value,
}

#[tool]
pub async fn fetch_data(url: String) -> ToolResponse {
    match make_http_request(&url).await {
        Ok(response) => ToolResponse::ok(serde_json::to_string(&response).unwrap()),
        Err(e) => ToolResponse::error(&format!("HTTP request failed: {}", e)),
    }
}

async fn make_http_request(url: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "FTL-Tool/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    let status = response.status().to_string();
    let data: serde_json::Value = response.json().await?;
    
    Ok(ApiResponse { status, data })
}

#[tool]
pub async fn post_json(url: String, payload: serde_json::Value) -> ToolResponse {
    let client = reqwest::Client::new();
    
    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status().as_u16();
            match response.text().await {
                Ok(body) => ToolResponse::ok(&format!("Status: {}, Body: {}", status, body)),
                Err(e) => ToolResponse::error(&format!("Failed to read response: {}", e)),
            }
        }
        Err(e) => ToolResponse::error(&format!("Request failed: {}", e)),
    }
}
```

### Python

Use the `requests` library or `httpx` for async requests:

```toml
# components/my-tool/pyproject.toml
[project]
dependencies = [
    "ftl-sdk",
    "httpx",
    "pydantic"
]
```

```python
# components/my-tool/src/__init__.py
from ftl_sdk import tool, ToolResponse
import httpx
import json
from typing import Dict, Any
from pydantic import BaseModel

class ApiResponse(BaseModel):
    status_code: int
    data: Dict[Any, Any]
    headers: Dict[str, str]

@tool
async def fetch_data(url: str) -> ToolResponse:
    """Fetch data from an external API."""
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.get(
                url,
                headers={"User-Agent": "FTL-Tool/1.0"}
            )
            
            api_response = ApiResponse(
                status_code=response.status_code,
                data=response.json() if response.headers.get("content-type", "").startswith("application/json") else {"text": response.text},
                headers=dict(response.headers)
            )
            
            return ToolResponse.ok(api_response.model_dump_json())
            
    except httpx.TimeoutException:
        return ToolResponse.error("Request timed out after 30 seconds")
    except httpx.HTTPStatusError as e:
        return ToolResponse.error(f"HTTP error {e.response.status_code}: {e.response.text}")
    except Exception as e:
        return ToolResponse.error(f"Request failed: {str(e)}")

@tool
async def post_json(url: str, payload: dict) -> ToolResponse:
    """Send JSON data to an external API."""
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(
                url,
                json=payload,
                headers={
                    "Content-Type": "application/json",
                    "User-Agent": "FTL-Tool/1.0"
                }
            )
            
            return ToolResponse.ok(f"Status: {response.status_code}, Response: {response.text}")
            
    except Exception as e:
        return ToolResponse.error(f"POST request failed: {str(e)}")

@tool
async def call_openai_api(prompt: str, api_key: str) -> ToolResponse:
    """Example: Call OpenAI's API."""
    try:
        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                "https://api.openai.com/v1/chat/completions",
                json={
                    "model": "gpt-3.5-turbo",
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": 150
                },
                headers={
                    "Authorization": f"Bearer {api_key}",
                    "Content-Type": "application/json"
                }
            )
            response.raise_for_status()
            
            result = response.json()
            message = result['choices'][0]['message']['content']
            return ToolResponse.ok(message)
            
    except httpx.HTTPStatusError as e:
        return ToolResponse.error(f"OpenAI API error {e.response.status_code}: {e.response.text}")
    except Exception as e:
        return ToolResponse.error(f"OpenAI request failed: {str(e)}")
```

### Go

Use the standard `net/http` package:

```go
// components/my-tool/main.go
//go:build wasip1

package main

import (
    "bytes"
    "context"
    "encoding/json"
    "fmt"
    "io"
    "log"
    "net/http"
    "time"

    "go.bytecodealliance.org/cm"
)

type ApiResponse struct {
    StatusCode int                 `json:"status_code"`
    Data       interface{}         `json:"data"`
    Headers    map[string]string   `json:"headers"`
}

type HttpToolImpl struct{}

func (h *HttpToolImpl) FetchData(ctx context.Context, url string) cm.Result[string, string] {
    client := &http.Client{
        Timeout: 30 * time.Second,
    }
    
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Failed to create request: %v", err))
    }
    req.Header.Set("User-Agent", "FTL-Tool/1.0")
    
    resp, err := client.Do(req)
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Request failed: %v", err))
    }
    defer resp.Body.Close()
    
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Failed to read response: %v", err))
    }
    
    // Parse JSON if content type indicates it
    var data interface{}
    contentType := resp.Header.Get("Content-Type")
    if len(contentType) > 0 && contentType[:16] == "application/json" {
        json.Unmarshal(body, &data)
    } else {
        data = string(body)
    }
    
    headers := make(map[string]string)
    for k, v := range resp.Header {
        if len(v) > 0 {
            headers[k] = v[0]
        }
    }
    
    response := ApiResponse{
        StatusCode: resp.StatusCode,
        Data:       data,
        Headers:    headers,
    }
    
    jsonBytes, _ := json.Marshal(response)
    return cm.OK[string](string(jsonBytes))
}

func (h *HttpToolImpl) PostJson(ctx context.Context, url string, payload string) cm.Result[string, string] {
    client := &http.Client{
        Timeout: 30 * time.Second,
    }
    
    req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewBufferString(payload))
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Failed to create request: %v", err))
    }
    
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("User-Agent", "FTL-Tool/1.0")
    
    resp, err := client.Do(req)
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Request failed: %v", err))
    }
    defer resp.Body.Close()
    
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return cm.Err[string](fmt.Sprintf("Failed to read response: %v", err))
    }
    
    result := fmt.Sprintf("Status: %d, Response: %s", resp.StatusCode, string(body))
    return cm.OK[string](result)
}

func main() {
    log.Println("HTTP tool component initialized")
}
```

### TypeScript/JavaScript

Use the `fetch` API (available in WASM environment):

```json
// components/my-tool/package.json
{
  "dependencies": {
    "ftl-sdk": "^0.1.0"
  }
}
```

```typescript
// components/my-tool/src/index.ts
import { tool, ToolResponse } from 'ftl-sdk';

interface ApiResponse {
  statusCode: number;
  data: any;
  headers: Record<string, string>;
}

@tool
export async function fetchData(url: string): Promise<ToolResponse> {
  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'FTL-Tool/1.0'
      }
    });

    const data = await response.json();
    const headers: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      headers[key] = value;
    });

    const apiResponse: ApiResponse = {
      statusCode: response.status,
      data,
      headers
    };

    return ToolResponse.ok(JSON.stringify(apiResponse));
  } catch (error) {
    return ToolResponse.error(`Request failed: ${error}`);
  }
}

@tool
export async function postJson(url: string, payload: object): Promise<ToolResponse> {
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'FTL-Tool/1.0'
      },
      body: JSON.stringify(payload)
    });

    const responseText = await response.text();
    return ToolResponse.ok(`Status: ${response.status}, Response: ${responseText}`);
  } catch (error) {
    return ToolResponse.error(`POST request failed: ${error}`);
  }
}
```

## Step 3: Build and Test

After implementing your HTTP client:

```bash
# Rebuild to apply configuration changes
ftl build

# Start the development server
ftl up

# Test your tool
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-api-tool/fetch_data",
    "arguments": {
      "url": "https://httpbin.org/json"
    }
  }'
```

## Common Patterns

### API Key Management

Store API keys as environment variables or configuration:

```toml
# ftl.toml
[tools.openai-tool]
path = "components/openai-tool"
allowed_outbound_hosts = ["https://api.openai.com"]
environment_variables = { "OPENAI_API_KEY" = "${OPENAI_API_KEY}" }
```

### Rate Limiting

Implement client-side rate limiting:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

static REQUEST_SEMAPHORE: once_cell::sync::Lazy<Arc<Semaphore>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Semaphore::new(10))); // 10 concurrent requests

#[tool]
pub async fn rate_limited_request(url: String) -> ToolResponse {
    let _permit = REQUEST_SEMAPHORE.acquire().await.unwrap();
    
    // Your HTTP request code here
    make_http_request(&url).await
}
```

### Retry Logic

Implement exponential backoff for transient failures:

```python
import asyncio
import random
from typing import Optional

async def http_request_with_retry(
    url: str, 
    max_retries: int = 3,
    base_delay: float = 1.0
) -> Optional[httpx.Response]:
    for attempt in range(max_retries + 1):
        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.get(url)
                response.raise_for_status()
                return response
        except (httpx.HTTPStatusError, httpx.TimeoutException) as e:
            if attempt == max_retries:
                raise e
            
            # Exponential backoff with jitter
            delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
            await asyncio.sleep(delay)
    
    return None
```

## Troubleshooting

### Common Issues

**"Connection refused" errors**:
- Verify the host is in `allowed_outbound_hosts`
- Check the URL format includes protocol (`https://`)
- Rebuild after configuration changes: `ftl build`

**Timeout errors**:
- Increase timeout values in your HTTP client
- Check if the external service is responding slowly
- Consider implementing retry logic

**SSL/TLS errors**:
- Ensure you're using `https://` for secure endpoints
- Some WASM environments have limited certificate stores

**JSON parsing errors**:
- Verify the API returns valid JSON
- Check `Content-Type` headers before parsing
- Handle non-JSON responses gracefully

### Debugging HTTP Requests

Enable detailed logging:

```rust
// Rust: Use env_logger
env_logger::init();
log::info!("Making request to: {}", url);
```

```python
# Python: Use logging
import logging
logging.basicConfig(level=logging.DEBUG)
```

### Testing External APIs

Use `httpbin.org` for testing HTTP functionality:

```bash
# Test GET request
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-tool/fetch_data",
    "arguments": {
      "url": "https://httpbin.org/json"
    }
  }'

# Test POST request  
curl -X POST http://localhost:3000/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-tool/post_json",
    "arguments": {
      "url": "https://httpbin.org/post",
      "payload": {"test": "data"}
    }
  }'
```

## Security Considerations

### Input Validation

Always validate URLs and payloads:

```rust
use url::Url;

fn validate_url(url_str: &str, allowed_hosts: &[&str]) -> Result<Url, String> {
    let url = Url::parse(url_str)
        .map_err(|_| "Invalid URL format")?;
    
    let host = url.host_str()
        .ok_or("URL missing host")?;
    
    if !allowed_hosts.iter().any(|&allowed| allowed.contains(host)) {
        return Err(format!("Host {} not allowed", host));
    }
    
    Ok(url)
}
```

### Sensitive Data

Never log sensitive information:

```python
# BAD: Logs API key
logging.info(f"Calling API with key: {api_key}")

# GOOD: Redacts sensitive data
logging.info(f"Calling API with key: {'*' * 8}")
```

## Next Steps

- **Authentication**: Learn to secure your MCP server in [Handling Authentication](./authentication.md)
- **Testing**: Write tests for your HTTP tools in [Testing Your Tools](./testing.md)  
- **Advanced Patterns**: Explore more complex examples in [Examples](../../examples/)
- **SDK Reference**: Check language-specific HTTP utilities in [SDK Reference](../sdk-reference/)

Making HTTP requests from FTL tools opens up integration with any web service or API, enabling powerful compositions of internal tools with external services.