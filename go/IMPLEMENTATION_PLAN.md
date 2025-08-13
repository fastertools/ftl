# 🎯 FTL Go Implementation Plan

## Current Status

### ✅ Completed
- Basic CLI structure with Cobra
- Shared libraries (config, spin executor) with 90%+ coverage
- Stub implementations for all commands
- Build system with Makefile
- Project initialization (3 templates)
- Basic build, up, deploy commands via Spin

### 🔴 Stub Implementations Found

#### 1. **Auth Commands** (`ftl/cmd/auth.go`)
- `ftl auth login` - "Auth login not yet implemented"
- `ftl auth logout` - "Auth logout not yet implemented" 
- `ftl auth status` - "Auth status not yet implemented"

#### 2. **Component Commands** (`ftl/cmd/component.go`)
- `ftl component add` - "Component add not yet implemented"
- `ftl component list` - "Component list not yet implemented"
- `ftl component remove` - "Component remove not yet implemented"

#### 3. **Test Command** (`ftl/cmd/test.go`)
- `ftl test` - "Test command not yet implemented"

#### 4. **Registry Commands** (`ftl/cmd/registry.go`)
- `ftl registry pull` - "Registry pull not yet implemented"
- `ftl registry list` - "Registry list not yet implemented"
- `ftl registry push` - ✅ Implemented (uses spin)

#### 5. **spin-compose** (`spin-compose/cmd/`)
- `spin-compose construct add` - "Construct addition not yet implemented"
- `spin-compose synth --env` - "Environment-specific configuration not yet implemented"

## 📋 Strategic Implementation Plan

### Phase 1: Authentication System (Priority: HIGH)
**Goal**: Enable users to authenticate with registries and FTL platform

#### 1.1 Create Auth Package (`shared/auth/`)
```go
// Core types
type Credentials struct {
    AuthKitDomain string
    AccessToken   string
    RefreshToken  string
    ExpiresAt     time.Time
}

// Storage interface
type CredentialStore interface {
    Load() (*Credentials, error)
    Save(*Credentials) error
    Delete() error
}

// Auth provider
type AuthProvider interface {
    Login(domain string) (*Credentials, error)
    Logout() error
    Refresh(creds *Credentials) (*Credentials, error)
    Status() (*AuthStatus, error)
}
```

#### 1.2 Implement Commands
- **Login**: OAuth device flow or browser-based auth
- **Logout**: Clear stored credentials
- **Status**: Show current auth state and token expiry
- **Storage**: Use OS keyring (keyring-go library) or encrypted file

**Reference**: Port logic from `/crates/commands/src/commands/auth.rs`

---

### Phase 2: Component Management (Priority: HIGH)
**Goal**: Manage WebAssembly components in projects

#### 2.1 Component Operations
```go
// Component types
type Component struct {
    Name     string
    Language string // rust, js, python, go
    Path     string
    Build    BuildConfig
}

type ComponentManager interface {
    Add(name, language string, opts AddOptions) error
    List() ([]Component, error)
    Remove(name string) error
    Build(name string) error
}
```

#### 2.2 Implementation Steps
- **Add**: 
  - Create component directory structure
  - Generate language-specific boilerplate
  - Update ftl.yaml and spinc.yaml
  - Support templates for each language
  
- **List**:
  - Read ftl.yaml/spinc.yaml
  - Parse component definitions
  - Display with status (built/not built)
  
- **Remove**:
  - Remove from config files
  - Optionally delete files
  - Clean build artifacts

**Reference**: `/crates/commands/src/commands/component.rs`

---

### Phase 3: Test Framework (Priority: MEDIUM)
**Goal**: Run component tests

#### 3.1 Test Runner
```go
type TestRunner interface {
    RunAll() error
    RunComponent(name string) error
    RunWithCoverage() error
}
```

#### 3.2 Implementation
- Detect test files per language
- Execute language-specific test commands
- Aggregate results
- Support watch mode

---

### Phase 4: Registry Operations (Priority: MEDIUM)
**Goal**: Complete registry interaction

#### 4.1 Registry Client
```go
type RegistryClient interface {
    Pull(reference string, dest string) error
    Push(source string, reference string) error
    List(registry string) ([]Component, error)
    Search(query string) ([]Component, error)
}
```

#### 4.2 Implementation
- **Pull**: Download components from registry
- **List**: Query registry for available components
- Use OCI registry standards
- Support authentication from Phase 1

---

### Phase 5: spin-compose Enhancements (Priority: LOW)
**Goal**: Complete infrastructure-as-code features

#### 5.1 Construct Management
- Add new constructs to project
- Template system for constructs
- Validation with CUE

#### 5.2 Environment Support
- Environment-specific overrides
- Variable substitution
- Secret management

---

## 🏗️ Implementation Order

### Week 1-2: Authentication
1. Create `shared/auth` package
2. Implement credential storage
3. Implement OAuth device flow
4. Add login/logout/status commands
5. Write tests (target 85%+ coverage)

### Week 3-4: Component Management
1. Create component manager
2. Implement add with templates
3. Implement list and remove
4. Update config file handlers
5. Write comprehensive tests

### Week 5: Testing & Registry
1. Implement test runner
2. Complete registry pull/list
3. Integration with auth system

### Week 6: Polish & Documentation
1. Improve error messages
2. Add progress indicators
3. Write user documentation
4. Integration tests

## 📦 Dependencies to Add

```go
// go/shared/go.mod
github.com/99designs/keyring       // Credential storage
github.com/pkg/browser             // Open browser for auth
golang.org/x/oauth2                // OAuth2 client
github.com/google/go-containerregistry // OCI registry client
```

## 🧪 Testing Strategy

1. **Unit Tests**: Each package with 85%+ coverage
2. **Integration Tests**: End-to-end command flows
3. **Mock Interfaces**: For external dependencies
4. **Test Fixtures**: Sample projects and components

## 📊 Success Metrics

- [ ] All stub implementations replaced
- [ ] 85%+ test coverage across all packages
- [ ] Authentication works with real registries
- [ ] Component lifecycle fully functional
- [ ] Compatible with existing Rust CLI configs
- [ ] Performance equal or better than Rust version

## 🔄 Migration Path

1. Feature parity with Rust CLI
2. Compatibility mode for existing projects
3. Migration tool for config updates
4. Parallel installation period
5. Deprecate and remove Rust version

## 📝 Notes from Rust Implementation

Key features to preserve:
- Interactive prompts with survey library
- Colored output and progress indicators
- Config file format compatibility
- Registry authentication flow
- Component publishing workflow
- Error message quality

## Next Immediate Steps

1. Start with auth package structure
2. Implement keyring-based credential storage
3. Create login command with device flow
4. Test with real registry authentication