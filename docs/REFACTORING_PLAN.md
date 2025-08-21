# FTL CLI Refactoring Plan

## Goal
Achieve GNU-level OSS quality by establishing a clean, CUE-centric architecture with no legacy cruft.

## Current Problems

1. **Dual validation paths**: Some commands use CUE, others use Go types
2. **No validation in critical paths**: Deploy bypasses CUE validation entirely
3. **Circular dependencies**: OCI package depends on types package
4. **Inconsistent data flow**: Multiple ways to parse and process configs
5. **Legacy artifacts**: pkg/types exists as intermediate layer

## Target Architecture

```
┌─────────────────┐
│   User Config   │ (YAML/JSON/CUE)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  CUE Validation │ (patterns.cue defines schema)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   CUE Value     │ (validated, typed, single source of truth)
└────────┬────────┘
         │
    ┌────┴────┬──────────┬────────┐
    ▼         ▼          ▼        ▼
┌──────┐ ┌────────┐ ┌────────┐ ┌─────┐
│Deploy│ │Scaffold│ │Component│ │Synth│
└──────┘ └────────┘ └────────┘ └─────┘
```

## Refactoring Steps

### Phase 1: Clean Package Boundaries

#### 1.1 Fix OCI Package (IMMEDIATE)
```go
// Before: pkg/oci depends on pkg/types
func (p *WASMPuller) Pull(ctx context.Context, source *ftltypes.RegistrySource) 

// After: pkg/oci is self-contained
func (p *WASMPuller) Pull(ctx context.Context, registry, pkg, version string)
```

#### 1.2 Create Core Validation (DONE)
- `internal/validation/validation.go` - CUE-based validation
- All configs flow through this pipeline
- Returns validated CUE values

### Phase 2: Update Commands

#### 2.1 Deploy Command
- Remove `loadDeployManifest()` using yaml.Unmarshal
- Use `validation.ValidateYAML/JSON/CUE()`
- Work with CUE values throughout
- Extract data using CUE paths

#### 2.2 Scaffold Command  
- Remove types.Manifest usage
- Use CUE for both reading and writing configs
- Validate after modifications

#### 2.3 Component Commands
- Update add/remove/list to use CUE
- Ensure validation on every change

### Phase 3: Remove Legacy

#### 3.1 Delete pkg/types
- No longer needed once all commands migrated
- OCI package self-contained
- Validation package handles all parsing

#### 3.2 Clean Comments
- Remove "legacy", "TODO", development artifacts
- Ensure all comments are meaningful

### Phase 4: Consolidate

#### 4.1 Single Validation Path
```go
// All commands use same pattern:
validated, err := validator.ValidateYAML(data)
if err != nil {
    return fmt.Errorf("validation failed: %w", err)
}
// Work with validated CUE value
```

#### 4.2 Consistent Error Handling
- CUE validation errors with context
- Show exact path that failed
- Actionable error messages

## Implementation Priority

1. **Fix OCI package** - Remove types dependency (CRITICAL)
2. **Migrate deploy** - Most important command
3. **Migrate scaffold** - Component generation
4. **Migrate component** - Add/remove operations
5. **Delete pkg/types** - Final cleanup

## Success Criteria

- [ ] Single validation pipeline through CUE
- [ ] No direct YAML/JSON unmarshaling to Go structs
- [ ] OCI package has no external dependencies
- [ ] All user input validated before processing
- [ ] Clean package boundaries
- [ ] No legacy comments or TODOs
- [ ] Comprehensive error messages
- [ ] Tests use CUE validation

## Migration Example

### Before (current deploy.go)
```go
func loadDeployManifest(configFile string) (*ftltypes.Manifest, error) {
    data, err := os.ReadFile(configFile)
    if err != nil {
        return nil, err
    }
    var manifest ftltypes.Manifest
    if err := yaml.Unmarshal(data, &manifest); err != nil {
        return nil, fmt.Errorf("failed to parse manifest: %w", err)
    }
    return &manifest, nil
}
```

### After (with CUE validation)
```go
func loadDeployManifest(configFile string) (cue.Value, error) {
    data, err := os.ReadFile(configFile)
    if err != nil {
        return cue.Value{}, err
    }
    
    validator := validation.New()
    validated, err := validator.ValidateYAML(data)
    if err != nil {
        return cue.Value{}, fmt.Errorf("validation failed: %w", err)
    }
    
    return validated, nil
}

// Usage:
name, _ := validated.LookupPath(cue.ParsePath("name")).String()
version, _ := validated.LookupPath(cue.ParsePath("version")).String()
```

## Code Quality Standards

1. **No unmarshaling without validation**
2. **CUE paths for data access**
3. **Explicit error handling**
4. **Clean package interfaces**
5. **Meaningful variable names**
6. **Comprehensive tests**

## Timeline

This is a pre-release refactor. We have freedom to:
- Break internal APIs
- Restructure packages
- Remove legacy code
- Establish clean patterns

Target: Complete refactor before initial release