# 🚀 FTL Go Implementation - Quick Reference

## Stub Implementations to Complete

### 🔴 High Priority (User-facing, frequently used)

| Command | File | Status | Rust Reference |
|---------|------|--------|----------------|
| `ftl auth login` | `ftl/cmd/auth.go:34` | ❌ Not implemented | `/crates/commands/src/commands/login.rs` |
| `ftl auth logout` | `ftl/cmd/auth.go:51` | ❌ Not implemented | `/crates/commands/src/commands/logout.rs` |
| `ftl auth status` | `ftl/cmd/auth.go:64` | ❌ Not implemented | `/crates/commands/src/commands/auth.rs` |
| `ftl component add` | `ftl/cmd/component.go:34` | ❌ Not implemented | `/crates/commands/src/commands/add.rs` |
| `ftl component list` | `ftl/cmd/component.go:47` | ❌ Not implemented | `/crates/commands/src/commands/component.rs` |
| `ftl component remove` | `ftl/cmd/component.go:62` | ❌ Not implemented | N/A - New feature |

### 🟡 Medium Priority

| Command | File | Status | Notes |
|---------|------|--------|-------|
| `ftl test` | `ftl/cmd/test.go:16` | ❌ Not implemented | Run component tests |
| `ftl registry pull` | `ftl/cmd/registry.go:58` | ❌ Not implemented | Download from registry |
| `ftl registry list` | `ftl/cmd/registry.go:71` | ❌ Not implemented | List registry contents |

### 🟢 Low Priority (spin-compose)

| Command | File | Status |
|---------|------|--------|
| `spin-compose construct add` | `spin-compose/cmd/construct.go:112` | ❌ Not implemented |
| `spin-compose synth --env` | `spin-compose/cmd/synth.go:66` | ❌ Not implemented |

## Implementation Checklist

### Phase 1: Auth System ⏱️ 2 weeks
- [ ] Create `shared/auth` package
- [ ] Implement credential storage (keyring)
- [ ] OAuth device flow
- [ ] Login command
- [ ] Logout command  
- [ ] Status command
- [ ] Tests with 85%+ coverage

### Phase 2: Components ⏱️ 2 weeks
- [ ] Component manager interface
- [ ] Add command with templates
- [ ] List command
- [ ] Remove command
- [ ] Config file updates
- [ ] Language templates (Rust, JS, Python, Go)
- [ ] Tests

### Phase 3: Testing & Registry ⏱️ 1 week
- [ ] Test runner implementation
- [ ] Registry pull command
- [ ] Registry list command
- [ ] Integration with auth

### Phase 4: Polish ⏱️ 1 week
- [ ] Progress indicators
- [ ] Better error messages
- [ ] Interactive prompts
- [ ] Documentation

## Key Files to Reference

### Rust Implementation (for logic reference)
```
/crates/commands/src/commands/
├── auth.rs         # Auth status logic
├── login.rs        # OAuth login flow
├── logout.rs       # Logout logic
├── add.rs          # Component add
├── component.rs    # Component publish/list
├── registry.rs     # Registry operations
└── test.rs         # Test runner
```

### Go Implementation (to complete)
```
/go/ftl/cmd/
├── auth.go         # 3 stubs to implement
├── component.go    # 3 stubs to implement
├── test.go         # 1 stub to implement
└── registry.go     # 2 stubs to implement

/go/shared/         # Add new packages here
├── auth/           # TO CREATE
├── component/      # TO CREATE
└── registry/       # TO CREATE
```

## Quick Test Commands

```bash
# Test current stubs
export PATH=$PATH:/home/ian/go/bin

ftl auth login      # Shows: "Auth login not yet implemented"
ftl auth status     # Shows: "Auth status not yet implemented"
ftl component add test --language rust  # Shows: "Component add not yet implemented"
ftl component list  # Shows: "Component list not yet implemented"
ftl test           # Shows: "Test command not yet implemented"
ftl registry list  # Shows: "Registry list not yet implemented"
```

## Dependencies to Add

```bash
# Add to go/shared/go.mod
go get github.com/99designs/keyring        # Credential storage
go get github.com/pkg/browser              # Browser launch
go get golang.org/x/oauth2                 # OAuth2
go get github.com/google/go-containerregistry  # OCI registry

# Add to go/ftl/go.mod  
go get github.com/AlecAivazis/survey/v2   # Interactive prompts
go get github.com/briandowns/spinner      # Progress indicators
```

## Success Criteria

✅ When complete, all commands should:
1. Work without showing "not yet implemented"
2. Have 85%+ test coverage
3. Match or exceed Rust CLI functionality
4. Provide clear error messages
5. Support both interactive and non-interactive modes

## Immediate Next Step

Start with auth system since many other features depend on it:

```bash
cd /home/ian/Dev/ftl-cli/go
mkdir -p shared/auth
# Create auth.go, storage.go, oauth.go
# Implement login command first
```