# Platform Team Update: FTL CLI v0.6.7-alpha.0

## Summary
We've released FTL CLI v0.6.7-alpha.0 with ECR registry support in the platform package. This resolves the "registry not allowed" errors you were experiencing.

## What's Fixed
- The `pkg/platform` package now includes FTL's ECR registry (795394005211.dkr.ecr.us-west-2.amazonaws.com) in the default allowed registries list
- Components hosted in our ECR registry will now be accepted alongside ghcr.io components

## How to Update

Update your Go module dependency:

```bash
go get github.com/fastertools/ftl-cli@v0.6.7-alpha.0
go mod tidy
```

## Updated Platform Package Usage

The platform package will now work correctly with your existing code:

```go
import "github.com/fastertools/ftl-cli/pkg/platform"

// DefaultConfig now includes both registries
config := platform.DefaultConfig()
// config.AllowedRegistries = ["ghcr.io", "795394005211.dkr.ecr.us-west-2.amazonaws.com"]

processor := platform.NewProcessor(config)
result, err := processor.Process(req)
```

## Verification

After updating, deployments with components from either registry will be accepted:
- ✅ ghcr.io (gateway, authorizer, and user components)  
- ✅ 795394005211.dkr.ecr.us-west-2.amazonaws.com (user components)

## No Code Changes Required

Your existing platform implementation doesn't need any changes. The registry whitelist is now configured correctly by default in the platform package.

## Release Notes
https://github.com/fastertools/ftl-cli/releases/tag/v0.6.7-alpha.0

Please update at your earliest convenience and let us know if you encounter any issues.