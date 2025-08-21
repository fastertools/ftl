# Release Commands for v0.8.0

## Pre-release Checklist

- [x] All tests passing
- [x] Documentation updated
- [x] CHANGELOG prepared
- [x] Migration guide written
- [x] go.mod dependencies resolved

## Release Steps

### 1. Run Final Tests
```bash
# Run all tests
go test ./...

# Run integration tests
go test -tags=integration ./pkg/policy/...

# Check coverage
go test ./pkg/policy/... -cover
go test ./pkg/platform/... -cover
```

### 2. Create Git Tag
```bash
# Ensure you're on main/master branch
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.8.0 -m "Release v0.8.0: Policy-based authorization with Rego

Major changes:
- Complete redesign of authorization system using Rego policies
- Support for user and machine tokens in org mode
- Custom policy support for advanced use cases
- 96% test coverage with real OPA evaluation

Breaking changes:
- Platform package internal restructuring (Process() API unchanged)
- Removed legacy authorization fields

See CHANGELOG_v0.8.0.md for full details."

# Push tag
git push origin v0.8.0
```

### 3. Create GitHub Release
```bash
# Using GitHub CLI
gh release create v0.8.0 \
  --title "v0.8.0: Policy-Based Authorization" \
  --notes-file CHANGELOG_v0.8.0.md \
  --prerelease=false
```

### 4. Publish Go Module
```bash
# Go modules are automatically available once tagged
# Verify it's available:
go list -m github.com/fastertools/ftl-cli@v0.8.0
```

### 5. Notify Platform Team

Send the following message to the platform backend team:

---

**Subject: FTL v0.8.0 Released - Authorization System Upgrade**

Hi Platform Team,

We've released FTL v0.8.0 with a complete redesign of the authorization system. The platform package now uses Rego policies for all authorization decisions.

**What you need to know:**
- Your existing code using `Process()` should work without changes
- The internals now generate Rego policies instead of using claim matching
- Both user and machine tokens are properly supported for org mode

**Action required:**
1. Update your dependency: `go get github.com/fastertools/ftl-cli@v0.8.0`
2. Run your test suite, especially auth-related tests
3. Test deployments with both user and machine actors

**Resources:**
- Migration Guide: `PLATFORM_MIGRATION.md`
- Full Changelog: `CHANGELOG_v0.8.0.md`
- New Authorization Docs: `docs/authorization.md`

The key benefit is that we now properly handle the WorkOS JWT schema differences (users without org_id, machines with org_id) through Rego policies rather than hacky Go code.

Please test thoroughly and let us know if you encounter any issues.

---

## Post-Release

### Monitor for Issues
- Watch GitHub issues for any problems
- Be ready to hotfix if needed

### Update Documentation Site
- Update any external documentation
- Add release notes to project website

### Version Bump for Development
```bash
# After release, bump version for development
git checkout -b post-v0.8.0-dev
# Update version references to v0.8.1-dev
git commit -m "chore: bump version to v0.8.1-dev"
git push origin post-v0.8.0-dev
```