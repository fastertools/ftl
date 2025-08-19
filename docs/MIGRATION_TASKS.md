# Types Package Migration Tasks

## Critical Path to Delete pkg/types

### 1. Deploy Command Migration
**File:** `internal/cli/deploy.go`
**Current State:** Still uses ftltypes.Manifest throughout
**Tasks:**
- [ ] Replace loadDeployManifest with CUE validation
- [ ] Use CUE paths instead of struct access
- [ ] Update processComponents to work with CUE values
- [ ] Update createDeploymentRequest to extract from CUE
- [ ] Fix displayDryRunSummary to use CUE values
- [ ] Test deployment end-to-end

### 2. Scaffold Command Migration  
**File:** `internal/scaffold/scaffold.go`
**Current State:** Uses types.Manifest for config updates
**Tasks:**
- [ ] Replace updateConfig to use CUE values
- [ ] Read configs through validation package
- [ ] Write configs maintaining CUE structure
- [ ] Update tests to use CUE validation

### 3. Component Commands Migration
**Files:** `internal/cli/component.go`, `component_add.go`, `component_remove.go`
**Current State:** All use types.Manifest
**Tasks:**
- [ ] Replace loadComponentManifest with CUE validation
- [ ] Update saveComponentManifest to preserve CUE
- [ ] Migrate list command to CUE paths
- [ ] Migrate add command to CUE manipulation
- [ ] Migrate remove command to CUE manipulation
- [ ] Update all component tests

### 4. Init Command Migration
**File:** `internal/cli/init.go`
**Current State:** Creates types.Manifest
**Tasks:**
- [ ] Generate CUE structure instead of Go struct
- [ ] Use CUE builder pattern
- [ ] Write validated CUE to file
- [ ] Update init tests

### 5. Platform Client Migration
**File:** `pkg/platform/client.go`
**Current State:** Uses types.ParseComponentSource
**Tasks:**
- [ ] Move ParseComponentSource logic inline
- [ ] Or create platform-specific types
- [ ] Remove types import

## Migration Strategy

### Phase 1: Deploy Command (HIGHEST PRIORITY)
This is the most critical command and needs immediate attention.

```go
// OLD WAY
manifest, err := loadDeployManifest(configFile)
appName := manifest.Application.Name

// NEW WAY  
validated, err := validator.ValidateYAML(data)
appName, _ := validated.LookupPath(cue.ParsePath("application.name")).String()
```

### Phase 2: Component Commands
These are frequently used and need careful migration.

```go
// OLD WAY
var manifest types.Manifest
yaml.Unmarshal(data, &manifest)

// NEW WAY
validated, err := validator.ValidateYAML(data)
components, _ := validated.LookupPath(cue.ParsePath("components")).List()
```

### Phase 3: Scaffold & Init
These create new configs and need CUE generation.

```go
// OLD WAY
manifest := &types.Manifest{
    Application: types.Application{Name: "app"},
}

// NEW WAY
cueStr := fmt.Sprintf(`
application: {
    name: %q
    version: "0.1.0"
}
components: []
`, appName)
validated := validator.ValidateCUE([]byte(cueStr))
```

### Phase 4: Platform Client
Either inline the parsing or create minimal platform types.

## Validation Before Deletion

Before deleting pkg/types:
1. Run `grep -r "pkg/types" .` - should return nothing
2. Run `go build ./...` - should compile
3. Run `go test ./...` - all tests should pass
4. Test actual deploy command with real app
5. Test scaffold creating new component
6. Test component add/remove

## Time Estimate

- Deploy command: 2-3 hours
- Component commands: 2 hours  
- Scaffold: 1-2 hours
- Init: 1 hour
- Platform client: 30 minutes
- Testing & validation: 2 hours

**Total: ~10 hours of focused work**

## The Hard Truth

We have:
- Created the architecture ✅
- Built the foundation ✅
- Fixed one critical dependency ✅

We have NOT:
- Actually migrated any commands ❌
- Integrated the validation package ❌
- Used CUE values anywhere ❌

The pkg/types package CANNOT be deleted until ALL migrations are complete.