# FTL CLI Release Notes - M2M Authentication & Claim Forwarding

## Summary
This release adds comprehensive Machine-to-Machine (M2M) authentication support and JWT claim forwarding capabilities to the FTL platform.

## Key Features

### 1. M2M Authentication for CLI
- **Environment Variable Support**: CLI auto-detects `FTL_CLIENT_ID` and `FTL_CLIENT_SECRET`
- **OAuth2 Client Credentials Flow**: Implements standard OAuth2 flow with WorkOS
- **Pre-generated Token Support**: Alternative authentication via `FTL_M2M_TOKEN`
- **Automatic Detection**: `ftl deploy` automatically uses M2M credentials when available
- **Organization Scoping**: M2M tokens are always associated with exactly one organization

### 2. Platform SDK Updates (`pkg/platform/client.go`)

#### New DeploymentContext Structure
```go
type DeploymentContext struct {
    ActorType     string            // "user" or "machine"
    OrgID         string            // Organization ID for org-scoped deployments
    ForwardClaims map[string]string // JWT claims to forward as headers
}
```

#### Enhanced ProcessRequest
The platform can now pass deployment context:
```go
req := platform.ProcessRequest{
    ConfigData: ftlConfig,
    Format: "yaml",
    AllowedSubjects: orgMembers,
    DeploymentContext: &platform.DeploymentContext{
        ActorType: "machine",
        OrgID: "org_01HQ3GXFP7",
        ForwardClaims: map[string]string{
            "sub": "X-User-ID",
            "org_id": "X-Org-ID",
            "email": "X-User-Email",
        },
    },
}
```

### 3. Automatic Security for M2M Deployments
- **Org-scoped Apps**: When a machine deploys to an org-scoped app, the platform automatically injects `mcp_auth_required_claims` with the org_id
- **JWT Validation**: The mcp-authorizer validates that M2M tokens contain the correct org_id claim
- **Isolation**: Ensures M2M tokens can only access their designated organization

### 4. JWT Claim Forwarding
- **Flexible Mapping**: Platform can specify which JWT claims to forward as HTTP headers
- **User Identification**: Forward `sub` claim as `X-User-ID` for audit trails
- **Organization Context**: Forward `org_id` as `X-Org-ID` for multi-tenant apps
- **Custom Claims**: Support for any JWT claim to header mapping

## Implementation Details

### Files Modified
- `internal/auth/m2m.go` - M2M authentication implementation
- `internal/auth/storage.go` - Extended credential store interface
- `internal/cli/auth.go` - Added `--machine` flag and token support
- `internal/cli/deploy.go` - Auto-detection of M2M credentials
- `pkg/platform/client.go` - Added DeploymentContext support
- `pkg/synthesis/patterns.cue` - Automatic claim injection for M2M

### Testing
✅ All tests passing (`go test ./...`)
✅ Code formatted (`go fmt ./...`)
✅ No linting issues (`golangci-lint run ./...`)
✅ Builds successfully (`go build ./...`)

## Usage Examples

### CI/CD Integration
```yaml
# GitHub Actions
env:
  FTL_CLIENT_ID: ${{ secrets.FTL_CLIENT_ID }}
  FTL_CLIENT_SECRET: ${{ secrets.FTL_CLIENT_SECRET }}

steps:
  - run: ftl deploy --yes  # Automatically uses M2M auth
```

### Manual M2M Login
```bash
# With client credentials
export FTL_CLIENT_ID=client_xxx
export FTL_CLIENT_SECRET=secret_xxx
ftl auth login --machine

# With pre-generated token
ftl auth login --machine --token="ey..."
```

## Backend Integration

The platform backend should:
1. Extract actor type from request headers (`X-FTL-Actor-Type`)
2. Extract org ID from headers (`X-FTL-Org-ID`)
3. Pass these to `ProcessRequest.DeploymentContext`
4. Optionally specify claims to forward for user component access

## Security Considerations
- M2M tokens are short-lived (typically 1 hour)
- Each M2M client is scoped to exactly one organization
- Automatic org_id claim validation prevents cross-org access
- All M2M deployments are tracked with actor type for audit

## Documentation
- `M2M_AUTHENTICATION.md` - Comprehensive architecture documentation
- `CLAUDE.md` - Updated with WASM constraints and M2M notes

## Ready for Production
This release is fully tested and ready for backend team integration. The platform API can immediately start using the new DeploymentContext features.