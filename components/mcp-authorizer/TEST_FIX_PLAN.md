# Test Fix Plan

## Root Cause Analysis

The tests are failing because:

1. **Configuration Schema Change**: The new implementation uses a cleaner configuration schema with `mcp_` prefixed variables
2. **JWKS Auto-derivation**: For AuthKit domains, JWKS URI is auto-derived but the endpoint still needs to be mocked
3. **Strict HTTPS Enforcement**: All URLs must be HTTPS (except internal gateway URLs)
4. **No Provider Type**: The new implementation doesn't use provider types - it's just JWT configuration

## Required Fixes

### 1. Test Environment Configuration
The `spin-test.toml` has been updated with the new schema.

### 2. Mock JWKS for AuthKit Tests
Tests that use AuthKit issuer need to mock the auto-derived JWKS endpoint:
```rust
// For issuer "https://test.authkit.app"
// Mock endpoint: "https://test.authkit.app/.well-known/jwks.json"
```

### 3. Configuration Updates
All test files have been updated to use:
- `mcp_auth_enabled` instead of `auth_enabled`
- `mcp_jwt_issuer` instead of `auth_provider_issuer`
- `mcp_jwt_jwks_uri` instead of `auth_provider_jwks_uri`
- `mcp_jwt_audience` instead of `auth_provider_audience`
- etc.

### 4. Remove Invalid Tests
Tests that check for invalid provider types should be removed or updated since we no longer have provider types.

### 5. Fix Gateway URL Mocks
All gateway URL mocks have been updated to use `http://test-gateway.spin.internal/mcp-internal`

## Implementation Status

✅ Configuration variable names updated in all test files
✅ spin-test.toml updated with new schema
✅ Gateway URL mocks updated
✅ Trace header support added to error responses
❌ JWKS endpoint mocking needs to be added where missing
❌ Some tests still expect old behavior and need updates

## Next Steps

1. Add JWKS mocking to tests that use AuthKit issuer without explicit JWKS URI
2. Update tests that expect configuration errors for scenarios that are now valid
3. Remove or update tests for removed features (provider types, etc.)