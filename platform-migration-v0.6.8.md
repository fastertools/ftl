# Platform API Migration Guide: FTL CLI v0.6.8-alpha.0

## Overview
We've simplified the platform package interface to create a cleaner separation of concerns between the platform API and FTL-specific logic. This is a breaking change that requires minor updates to your integration.

## Key Changes

### 🎯 Simplified Interface
The `platform.Process()` method now has a minimal, focused interface:
- **Input**: Raw FTL config + computed allowed subjects
- **Output**: Deployable Spin TOML + metadata
- **No longer returns**: Application struct (not needed by platform)

### 📦 What Changed

#### Before (v0.6.7)
```go
result, err := processor.Process(ProcessRequest{
    ConfigData: ftlYAML,
    Format: "yaml",
    Variables: deployVars,          // REMOVED - not used
    AllowedSubjects: subjects,
})
// Could access: result.Application.Name, result.Application.Components, etc.
```

#### After (v0.6.8)
```go
result, err := processor.Process(ProcessRequest{
    ConfigData: ftlYAML,
    Format: "yaml",
    AllowedSubjects: subjects,  // Only needed for private/org modes
})
// Returns: result.SpinTOML (ready to deploy) + result.Metadata (for logging)
```

## Migration Steps

### 1. Update your imports
```go
import "github.com/fastertools/ftl-cli/pkg/platform"
```

### 2. Update Process() calls
Remove any code that accesses `result.Application` - it's no longer returned or needed.

### 3. Handle allowed subjects correctly

#### For `public` access mode:
```go
// No allowed subjects needed
result, err := processor.Process(ProcessRequest{
    ConfigData: configData,
    Format: "yaml",
    AllowedSubjects: nil,  // or empty slice
})
```

#### For `private` access mode:
```go
// Provide the single authenticated user
result, err := processor.Process(ProcessRequest{
    ConfigData: configData,
    Format: "yaml",
    AllowedSubjects: []string{authenticatedUserID},
})
```

#### For `org` access mode:
```go
// 1. Get org members from WorkOS
orgMembers := workosClient.GetOrgMembers(orgID)

// 2. If the FTL config specifies allowed_roles, filter the members
// (The platform package will tell you via metadata if roles were specified)

// 3. Pass the filtered subjects
result, err := processor.Process(ProcessRequest{
    ConfigData: configData,
    Format: "yaml",
    AllowedSubjects: orgMemberIDs,
})
```

#### For `custom` access mode:
```go
// No allowed subjects needed (app handles its own auth)
result, err := processor.Process(ProcessRequest{
    ConfigData: configData,
    Format: "yaml",
    AllowedSubjects: nil,
})
```

### 4. Use the metadata
The `ProcessResult.Metadata` now includes:
```go
type ProcessMetadata struct {
    AppName            string  // For logging/tracking
    AppVersion         string  
    ComponentCount     int     
    AccessMode         string  // "public", "private", "org", "custom"
    InjectedGateway    bool    
    InjectedAuthorizer bool    // true for private/org modes
    SubjectsInjected   int     // Number of subjects actually injected
}
```

## Benefits of This Change

1. **Cleaner Separation**: Platform handles WorkOS/user management, FTL package handles all FTL-specific logic
2. **Simpler Interface**: Just pass config + subjects, get back deployable TOML
3. **Less Coupling**: Platform doesn't need to understand FTL schema or structure
4. **Easier Testing**: Simpler interface means easier mocking and testing

## Example Integration

```go
func (s *PlatformService) DeployApp(ctx context.Context, appID, configYAML string, user User) error {
    // 1. Determine access mode (parse YAML minimally or store separately)
    accessMode := s.getAccessMode(configYAML)
    
    // 2. Compute allowed subjects based on mode
    var allowedSubjects []string
    switch accessMode {
    case "private":
        allowedSubjects = []string{user.ID}
    case "org":
        allowedSubjects = s.getOrgMembers(user.OrgID)
        // Note: If config has allowed_roles, you should filter here
    }
    
    // 3. Process with FTL package
    processor := platform.NewProcessor(platform.DefaultConfig())
    result, err := processor.Process(platform.ProcessRequest{
        ConfigData:      []byte(configYAML),
        Format:         "yaml",
        AllowedSubjects: allowedSubjects,
    })
    if err != nil {
        return fmt.Errorf("FTL processing failed: %w", err)
    }
    
    // 4. Deploy the Spin TOML
    return s.deployToSpinPlatform(result.SpinTOML)
}
```

## Questions or Issues?

If you have any questions about this migration or encounter any issues:
1. Check the updated tests in `pkg/platform/client_test.go` for examples
2. Open an issue at https://github.com/fastertools/ftl-cli/issues
3. The change is backward-incompatible but the migration is straightforward

## Version
This change is effective in `v0.6.8-alpha.0` and later versions.