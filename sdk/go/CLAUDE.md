# FTL Go SDK - Important Build Information

## Known Issues

### Spin SDK Build Error
**CRITICAL**: The Spin Go SDK has a known issue with export comments that causes compilation errors:
```
# github.com/spinframework/spin-go-sdk/http
../../../../../go/pkg/mod/github.com/spinframework/spin-go-sdk@v0.0.0-20250411015808-ee0bd1e7d170/http/internals.go:16:1: export comment has wrong name "spin_http_handle_http_request", want "handle_http_request"
```

This is a **KNOWN ISSUE** that has been present for a while and affects all builds that import the Spin HTTP package.

## Required Build Tags

To work around this issue, **ALWAYS USE BUILD TAGS** when testing the SDK:

### For Running Tests
```bash
# Run tests with the 'test' build tag to use stub implementations
go test -tags test ./...
```

### For Building Production Code
```bash
# Build without test tag to use real Spin HTTP
go build -tags '!test' ./...
```

## File Structure

The SDK uses build tags to separate test and production code:

- `handlers_v3_http.go` - Production HTTP handler (build tag: `!test`)
- `handlers_v3_test_stub.go` - Test stub implementation (build tag: `test`)
- Test files (`*_test.go`) - Should be run with `-tags test`

## Important Notes

1. **DO NOT** try to fix the Spin SDK export comment issue - it's upstream
2. **DO NOT** try to build without proper build tags
3. **ALWAYS** use `-tags test` when running tests
4. **REMEMBER** this issue exists and use the workaround

## Testing Commands

```bash
# Run all tests
go test -tags test ./...

# Run specific test
go test -tags test -run TestName ./...

# Run with verbose output
go test -tags test -v ./...
```

## Why This Matters

The V3 SDK implementation uses conditional compilation to:
- Avoid Spin HTTP dependencies during testing
- Provide stub implementations for unit tests
- Allow the SDK to compile and test despite upstream issues

This approach ensures the SDK remains testable and maintainable while the upstream issue is unresolved.