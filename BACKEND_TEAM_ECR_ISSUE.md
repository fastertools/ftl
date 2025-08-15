# ECR Registry Naming Issue - CLI/Backend Integration

**To:** Platform Backend Team  
**From:** CLI Team  
**Date:** August 15, 2025  
**Subject:** Critical Issue: ECR Repository Naming Incompatible with Spin Deps

## Executive Summary

We've discovered a blocking issue with the ECR registry integration. The UUID-based repository naming scheme is incompatible with `spin deps publish` command requirements, preventing component deployment.

## The Problem

When attempting to deploy a component, we get this error:

```bash
$ ftl deploy
...
ℹ Pushing component 'graph' to 795394005211.dkr.ecr.us-west-2.amazonaws.com/9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e:graph@0.1.0

Error: failed to push components: failed to push component graph: 
error: invalid value '9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e:graph@0.1.0' for '--package <PACKAGE>': 
invalid label: dash-separated words must begin with an ASCII lowercase letter
```

## Technical Details

### Current Flow

1. **Backend creates app** with UUID: `9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e`
2. **Backend returns ECR credentials** with:
   - Registry URI: `795394005211.dkr.ecr.us-west-2.amazonaws.com`
   - App uses UUID as repository/namespace
3. **CLI attempts to push** using:
   ```bash
   spin deps publish \
     --registry 795394005211.dkr.ecr.us-west-2.amazonaws.com \
     --package 9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e:graph@0.1.0 \
     ./dist/graph.wasm
   ```
4. **Spin deps rejects** the package name because UUIDs start with numbers

### Spin Deps Requirements

The `spin deps publish` command expects package names in format: `<namespace>:<name>@<version>`

Where:
- **namespace** must start with an ASCII lowercase letter
- Dash-separated words must each begin with lowercase letters
- Numbers are allowed, but not at the start

Valid examples:
- ✅ `ftl-apps:graph@0.1.0`
- ✅ `app-9423dfaa:graph@0.1.0`
- ❌ `9423dfaa-2bd8-43fa:graph@0.1.0` (starts with number)

## Proposed Solutions

### Option 1: Use App Name as Namespace (Recommended)

**Backend changes:**
- Use the app's name (e.g., "math") as the ECR repository namespace
- Handle potential naming conflicts with a suffix if needed

**Example:**
```bash
# Instead of UUID-based:
9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e:graph@0.1.0

# Use name-based:
math:graph@0.1.0
# or if conflict:
math-9423dfaa:graph@0.1.0
```

**Pros:**
- Clean, readable package names
- Compatible with spin deps
- Maintains relationship between app name and packages

**Cons:**
- Need to handle name uniqueness
- Existing UUID-based repos need migration

### Option 2: Provide Package Name Mapping

**Backend changes:**
- Add a `packageNamespace` field to the ECR token response
- Map UUID to a valid package namespace server-side

**Response example:**
```json
{
  "registryUri": "795394005211.dkr.ecr.us-west-2.amazonaws.com",
  "authorizationToken": "...",
  "appId": "9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e",
  "packageNamespace": "app-9423dfaa"  // New field
}
```

**Pros:**
- Backend maintains full control
- No changes to existing ECR structure
- Backward compatible

**Cons:**
- Additional mapping layer
- Divergence between ECR repo name and package name

### Option 3: Use Docker/OCI Push Instead

**CLI changes:**
- Skip `spin deps publish` entirely
- Use docker/buildah/skopeo to push directly to ECR

**Example:**
```bash
# Convert WASM to OCI image and push directly
docker tag graph.wasm:latest \
  795394005211.dkr.ecr.us-west-2.amazonaws.com/9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e/graph:0.1.0
docker push ...
```

**Pros:**
- Full control over repository structure
- No spin deps limitations

**Cons:**
- Requires docker/buildah dependency
- More complex than spin deps
- May not preserve WASM component metadata correctly

### Option 4: Single Namespace with Prefixed Packages

**Backend changes:**
- Use a single ECR repository for all apps
- Prefix package names with sanitized app identifier

**Example:**
```bash
# Single namespace "ftl-apps"
ftl-apps:math-graph@0.1.0
ftl-apps:todoapp-api@1.0.0
```

**Pros:**
- Simple ECR structure
- All packages in one place
- Easy browsing/discovery

**Cons:**
- Potential scaling issues
- Less isolation between apps
- Complex ACL management

## Impact Analysis

### Current State
- ❌ Deployments are completely blocked
- ❌ Cannot test end-to-end flow
- ❌ Cannot validate synthesis with real components

### What's Working
- ✅ App creation succeeds
- ✅ ECR authentication works
- ✅ Local component builds work
- ✅ FTL configuration synthesis works

## Recommended Action

We recommend **Option 1** (Use App Name as Namespace) because:
1. It's the most intuitive for users
2. Aligns with spin/warg ecosystem conventions
3. Provides clean package names
4. Minimal backend changes required

## Quick Fix for Testing

For immediate unblocking, could you provide a test app with a compliant namespace? For example:
- App ID: `9423dfaa-2bd8-43fa-bcd9-b0599ea44d5e`
- Package namespace: `testapp` or `ftl-test`

This would let us complete end-to-end testing while you implement the permanent solution.

## Questions for Backend Team

1. Are there constraints on ECR repository naming we should consider?
2. Is the UUID-based naming critical for other parts of the system?
3. Would you prefer to handle sanitization server-side or client-side?
4. Can we coordinate on a migration strategy for existing apps?

## Next Steps

1. **Immediate:** Provide test namespace for unblocking
2. **Short-term:** Agree on solution approach
3. **Implementation:** Update backend ECR provisioning
4. **Testing:** Validate end-to-end deployment flow
5. **Migration:** Handle existing UUID-based repositories

## Code Context

Here's where the issue occurs in our code:

```go
// go/ftl/cmd/deploy.go:501-504
cmd := exec.CommandContext(ctx, "spin", "deps", "publish",
    "--registry", registryURI,
    "--package", fmt.Sprintf("%s:%s@%s", appID, comp.ID, version),  // <-- appID is UUID
    sourcePath)
```

## We're Here to Help

This is a solvable problem, and we're happy to:
- Adjust the CLI implementation to match your chosen approach
- Help test the solution
- Coordinate on timeline and rollout

Please let us know your thoughts and preferred solution. We're standing by to help implement whatever approach works best for the platform.

Best regards,  
The CLI Team

---

**P.S.** The shared ftl package integration is working beautifully otherwise! The synthesis and type consistency are perfect. This is the last blocker for full end-to-end deployment.