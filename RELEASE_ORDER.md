# Rust SDK Release Order

## Current Situation
- **ftl-sdk-macros** latest on crates.io: **0.11.1**
- **ftl-sdk-macros** in repo: **0.13.0** (never published)
- **ftl-sdk** depends on: `^0.11.1` (now fixed)

## Release Order (IMPORTANT!)

### Step 1: Release ftl-sdk-macros FIRST
1. Wait for PR #347 (ftl-sdk-macros v0.14.0) to rebuild with the fix
2. Once CI passes, **merge PR #347**
3. GitHub Actions will automatically publish to crates.io
4. Verify: `curl -s https://crates.io/api/v1/crates/ftl-sdk-macros | jq '.versions[0].num'`

### Step 2: Update ftl-sdk dependency
1. Create a new PR to update ftl-sdk:
   ```bash
   git checkout -b update-macros-dep
   # Edit sdk/rust/Cargo.toml
   # Change: ftl-sdk-macros = { version = "^0.11.1", optional = true }
   # To:     ftl-sdk-macros = { version = "^0.14", optional = true }
   git add sdk/rust/Cargo.toml
   git commit -m "chore: update ftl-sdk-macros dependency to ^0.14"
   git push origin update-macros-dep
   gh pr create --title "chore: update ftl-sdk-macros dependency" --body "Updates to use the newly published 0.14.0 version"
   ```

### Step 3: Release ftl-sdk
1. After the dependency update PR is merged
2. The ftl-sdk release PR #346 will be updated by release-please
3. Once CI passes, merge PR #346
4. GitHub Actions publishes ftl-sdk to crates.io

## Why This Order?

- **Dependencies must exist on crates.io** before dependents can use them
- CI will fail if it tries to build against non-existent crate versions
- This is exactly how it would work with separate repositories

## For Future Releases

Once both are at 0.14.0 on crates.io:
- Minor/patch updates to ftl-sdk-macros (0.14.1, 0.14.2) will work automatically
- Major updates (0.15.0) will need a dependency update PR for ftl-sdk
- This is standard Rust/Cargo versioning practice# Triggering release-please
