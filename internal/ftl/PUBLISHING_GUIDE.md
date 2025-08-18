# Publishing the FTL Shared Package

## Quick Start for Backend Team

Until we publish to a registry, you have three options to use this package:

### Option 1: Git Submodule (Recommended for now)

```bash
# In your backend repository
git submodule add https://github.com/fastertools/ftl-cli.git vendor/ftl-cli
git submodule update --init --recursive

# In your go.mod
replace github.com/fastertools/ftl-cli/go/shared/ftl => ./vendor/ftl-cli/go/shared/ftl
```

### Option 2: Direct GitHub Import

```bash
# This will work once we tag a release
go get github.com/fastertools/ftl-cli/go/shared/ftl@v0.1.0
```

### Option 3: Local Development

```bash
# Clone the repo locally
git clone https://github.com/fastertools/ftl-cli.git /path/to/ftl-cli

# In your go.mod
replace github.com/fastertools/ftl-cli/go/shared/ftl => /path/to/ftl-cli/go/shared/ftl
```

## Publishing Strategy

### Phase 1: Multi-Module Repository (Current)

The FTL CLI repository contains multiple Go modules:

```
ftl-cli/
├── go.mod                    # Root module (optional)
├── go/
│   ├── ftl/
│   │   └── go.mod           # CLI module
│   └── shared/
│       ├── api/
│       │   └── go.mod       # API client module
│       ├── auth/
│       │   └── go.mod       # Auth module
│       └── ftl/
│           └── go.mod       # Shared FTL module (THIS ONE)
```

### Phase 2: Tagged Releases

We'll use Go's module versioning with git tags:

```bash
# Tag the shared module specifically
git tag go/shared/ftl/v0.1.0
git push origin go/shared/ftl/v0.1.0

# Backend team can then import
go get github.com/fastertools/ftl-cli/go/shared/ftl@v0.1.0
```

### Phase 3: Separate Module (Future)

Eventually, we might extract this to its own repository:

```
github.com/fastertools/ftl-sdk-go
├── go.mod
├── types.go
├── synthesis.go
├── deployment.go
└── patterns.cue
```

## Immediate Steps for Publishing

### 1. Update the go.mod with proper module path

```go
module github.com/fastertools/ftl-cli/go/shared/ftl

go 1.21

require (
    cuelang.org/go v0.7.0
    gopkg.in/yaml.v3 v3.0.1
)
```

### 2. Create a release workflow

`.github/workflows/release-shared-ftl.yml`:

```yaml
name: Release Shared FTL Module

on:
  push:
    tags:
      - 'go/shared/ftl/v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions/setup-go@v4
        with:
          go-version: '1.21'
      
      - name: Test module
        working-directory: go/shared/ftl
        run: |
          go mod tidy
          go test ./...
      
      - name: Create Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: FTL Shared Module ${{ github.ref }}
          body: |
            Shared FTL types and synthesis for CLI and backend
            
            ## Usage
            ```go
            import "github.com/fastertools/ftl-cli/go/shared/ftl"
            ```
          draft: false
          prerelease: false
```

### 3. Version compatibility matrix

| FTL CLI Version | Shared Module Version | Backend Compatible |
|----------------|-----------------------|-------------------|
| v1.0.0         | v0.1.0               | ✅                |
| v1.1.0         | v0.2.0               | ✅                |

## For Backend Team: Immediate Usage

### Step 1: Add to go.mod

```go
module your-backend

go 1.21

require (
    github.com/fastertools/ftl-cli/go/shared/ftl v0.0.0-20240101000000-abcdef123456
)

// Temporary: Use replace directive until we tag releases
replace github.com/fastertools/ftl-cli/go/shared/ftl => github.com/fastertools/ftl-cli/go/shared/ftl@main
```

### Step 2: go get from GitHub

```bash
# Get the latest from main branch (temporary)
go get github.com/fastertools/ftl-cli/go/shared/ftl@main

# Or specific commit
go get github.com/fastertools/ftl-cli/go/shared/ftl@6bb943e

# Once we tag releases
go get github.com/fastertools/ftl-cli/go/shared/ftl@v0.1.0
```

### Step 3: Import and use

```go
import (
    "github.com/fastertools/ftl-cli/go/shared/ftl"
)

func handleDeployment(req ftl.DeploymentRequest) error {
    manifest, err := ftl.ProcessDeploymentRequest(&req)
    // ...
}
```

## Versioning Strategy

### Semantic Versioning

- **Patch** (v0.1.1): Bug fixes, no API changes
- **Minor** (v0.2.0): New features, backward compatible
- **Major** (v1.0.0): Breaking API changes

### Breaking Change Policy

When we need breaking changes:

1. Announce in advance
2. Support both versions temporarily
3. Provide migration guide
4. Coordinate CLI and backend updates

### Version Pinning

Backend should pin to specific versions:

```go
require (
    github.com/fastertools/ftl-cli/go/shared/ftl v0.1.0
)
```

## Testing Integration

### Backend Integration Test

```go
// integration_test.go
package main

import (
    "testing"
    "github.com/fastertools/ftl-cli/go/shared/ftl"
)

func TestSharedPackageImport(t *testing.T) {
    app := &ftl.Application{
        Name:    "test",
        Version: "1.0.0",
    }
    
    app.SetDefaults()
    err := app.Validate()
    if err != nil {
        t.Fatalf("Failed to validate: %v", err)
    }
    
    synth := ftl.NewSynthesizer()
    manifest, err := synth.SynthesizeToSpin(app)
    if err != nil {
        t.Fatalf("Failed to synthesize: %v", err)
    }
    
    if manifest.Application.Name != "test" {
        t.Errorf("Expected app name 'test', got %s", manifest.Application.Name)
    }
}
```

Run:
```bash
go test -v
```

## CI/CD Integration

### Backend CI Pipeline

```yaml
# .github/workflows/backend-ci.yml
name: Backend CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions/setup-go@v4
        with:
          go-version: '1.21'
      
      - name: Get FTL shared module
        run: |
          go get github.com/fastertools/ftl-cli/go/shared/ftl@main
          go mod tidy
      
      - name: Test
        run: go test ./...
      
      - name: Build
        run: go build ./...
```

## Coordination Process

### Release Coordination

1. **FTL CLI Team**:
   - Makes changes to shared package
   - Tests locally
   - Creates PR with changes
   - Tags release after merge

2. **Backend Team**:
   - Reviews changes
   - Tests with new version
   - Updates go.mod
   - Deploys with new version

### Communication

- **Breaking changes**: Announced in #ftl-platform Slack
- **New releases**: Automated notification via GitHub
- **Version compatibility**: Tracked in this document

## Emergency Procedures

### Rollback

If a version causes issues:

```bash
# Pin to previous working version
go get github.com/fastertools/ftl-cli/go/shared/ftl@v0.0.9
go mod tidy
```

### Hotfix

For urgent fixes:

1. Create hotfix branch
2. Fix issue
3. Tag as patch version
4. Backend updates immediately

## FAQ

**Q: Can we use this before official release?**
A: Yes, use the GitHub URL with @main or specific commit

**Q: How do we handle CUE pattern updates?**
A: The patterns.cue is embedded, so updates require new module version

**Q: What about private repositories?**
A: Configure GOPRIVATE environment variable or use SSH URLs

**Q: Can we vendor the dependency?**
A: Yes, use `go mod vendor` to include in your repository

## Next Steps

1. **Immediate**: Backend team uses GitHub import with @main
2. **This week**: Tag first official release v0.1.0
3. **Next sprint**: Set up automated releases
4. **Future**: Consider separate SDK repository