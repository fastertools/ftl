# FTL CLI Refactoring Status

## Completed ✅

### 1. Architecture Design
- Created comprehensive architecture documentation
- Defined CUE-centric validation pipeline
- Established clear package boundaries

### 2. OCI Package Refactoring
**Before:**
```go
func (p *WASMPuller) Pull(ctx context.Context, source *ftltypes.RegistrySource) (string, error)
```

**After:**
```go
func (p *WASMPuller) Pull(ctx context.Context, registry, packageName, version string) (string, error)
```

- Removed dependency on `pkg/types`
- Self-contained package with clean API
- All tests updated and passing

### 3. Validation Package Created
- `internal/validation/validation.go` implements CUE-based validation
- Validates YAML/JSON/CUE through single pipeline
- Returns validated CUE values for type-safe access

### 4. Synthesizer Enhanced
- Exposed `GetPatterns()` for validation package
- Schema available for validation pipeline

## In Progress 🚧

### Deploy Command Refactoring
- Created `deploy_new.go` with CUE validation
- Need to replace old `deploy.go`
- Update to use new OCI API

## Pending 📋

### 1. Update Remaining Commands
- **Scaffold**: Remove types.Manifest usage
- **Component**: Update add/remove/list operations
- **Init**: Use CUE validation

### 2. Remove Legacy Code
- Delete `pkg/types` package
- Clean up old comments
- Remove development artifacts

### 3. Update Integration Points
- Fix deploy command calls to OCI package
- Update scaffold to use CUE values
- Ensure all paths use validation

## Migration Checklist

### Deploy Command
- [x] Create new validation-based implementation
- [ ] Update OCI package calls with new signature
- [ ] Test with real deployments
- [ ] Replace old implementation

### Scaffold Command
- [ ] Replace types.Manifest with CUE values
- [ ] Use validation package for config updates
- [ ] Test component generation

### Component Commands
- [ ] Update add command
- [ ] Update remove command
- [ ] Update list command

### Cleanup
- [ ] Delete pkg/types
- [ ] Remove legacy comments
- [ ] Update all imports
- [ ] Run full test suite

## Key Changes Summary

1. **OCI Package**: Now uses explicit parameters instead of types
2. **Validation**: All configs flow through CUE validation
3. **Data Access**: Use CUE paths instead of Go structs
4. **Error Handling**: Validation errors with context

## Next Steps

1. Complete deploy command migration
2. Update all commands to use validation
3. Remove pkg/types entirely
4. Full integration testing