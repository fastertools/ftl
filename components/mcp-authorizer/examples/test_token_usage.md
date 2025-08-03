# Test Token Generation Utilities

The MCP Authorizer includes test utilities to help with JWT token generation in tests. These utilities make it easy to create valid JWT tokens without complex setup.

## Basic Usage

```rust
use ftl_mcp_authorizer::test_utils::{TestKeyPair, create_test_token};

// Generate a test key pair
let key_pair = TestKeyPair::generate();

// Get the public key in PEM format for configuration
let public_key_pem = key_pair.public_key_pem();

// Create a simple test token with scopes
let token = create_test_token(&key_pair, vec!["read", "write"]);
```

## Advanced Token Building

The `TestTokenBuilder` provides a fluent API for creating customized tokens:

```rust
use ftl_mcp_authorizer::test_utils::{TestKeyPair, TestTokenBuilder};
use chrono::Duration;

let key_pair = TestKeyPair::generate();

let token = key_pair.create_token(
    TestTokenBuilder::new()
        .subject("user-123")
        .issuer("https://auth.example.com")
        .audience("https://api.example.com")
        .scopes(vec!["admin", "api"])
        .client_id("my-app")
        .expires_in(Duration::hours(2))
        .kid("test-key-1")
        .claim("department", serde_json::json!("engineering"))
);
```

## Microsoft-style Claims

Support for Microsoft's `scp` claim format:

```rust
// As a space-separated string
let token = key_pair.create_token(
    TestTokenBuilder::new()
        .scp_string("user.read mail.read")
);

// As an array
let token = key_pair.create_token(
    TestTokenBuilder::new()
        .scp_array(vec!["user.read", "mail.read"])
);
```

## Multiple Audiences

```rust
let token = key_pair.create_token(
    TestTokenBuilder::new()
        .audiences(vec![
            "https://api1.example.com".to_string(),
            "https://api2.example.com".to_string(),
        ])
);
```

## Expired Tokens

For testing token expiration:

```rust
use ftl_mcp_authorizer::test_utils::create_expired_token;

let expired_token = create_expired_token(&key_pair);
```

## Complete Test Example

```rust
#[test]
fn test_jwt_authentication() {
    use ftl_mcp_authorizer::test_utils::{TestKeyPair, TestTokenBuilder};
    
    // Generate keys
    let key_pair = TestKeyPair::generate();
    
    // Configure your JWT provider with the test public key
    configure_jwt_provider(&key_pair.public_key_pem());
    
    // Create a token with required scopes
    let token = key_pair.create_token(
        TestTokenBuilder::new()
            .issuer("https://test.issuer.com")
            .scopes(vec!["admin", "write"])
    );
    
    // Use the token in your test
    let response = make_authenticated_request(&token);
    assert_eq!(response.status(), 200);
}
```

## Key Features

- **Easy RSA key generation**: `TestKeyPair::generate()` creates 2048-bit RSA keys
- **Fluent API**: Chain methods to build tokens with exactly the claims you need
- **Standards compliant**: Generates valid JWT tokens with RS256 algorithm
- **Flexible claims**: Support for both OAuth2 `scope` and Microsoft `scp` claims
- **Test helpers**: Pre-built functions for common scenarios (expired tokens, etc.)

## Security Note

These utilities are designed for testing only. Never use test-generated keys in production environments.