# Rust SDK Release Process (KISS)

## Overview

The Rust SDK packages (`ftl-sdk` and `ftl-sdk-macros`) are treated as **completely independent packages** that happen to live in the same repository. This is the simplest approach and mirrors how it would work if they were in separate repos.

## How It Works

### 1. Independent Releases
- `ftl-sdk-macros` gets its own release PR when it has changes
- `ftl-sdk` gets its own release PR when it has changes  
- They can be released at different times, different versions
- No complex coordination needed

### 2. Dependency Management
- `ftl-sdk` depends on `ftl-sdk-macros` with a normal semver range: `^0.13`
- This means it works with any 0.13.x version of ftl-sdk-macros
- When ftl-sdk-macros releases 0.14.0 with breaking changes, we manually update ftl-sdk in a normal PR

### 3. Release Order (when both need releases)
1. Merge the `ftl-sdk-macros` release PR first
2. Wait for it to publish to crates.io (automated via GitHub Actions)
3. Merge the `ftl-sdk` release PR
4. It will use the already-published macros version from crates.io

## Common Scenarios

### Scenario 1: Bug fix in ftl-sdk-macros
1. Fix lands in main
2. Release-please creates PR for ftl-sdk-macros (e.g., 0.13.1)
3. Merge it
4. ftl-sdk automatically uses 0.13.1 (due to ^0.13 range)
5. No ftl-sdk release needed!

### Scenario 2: Breaking change in ftl-sdk-macros
1. Breaking change lands in main
2. Release-please creates PR for ftl-sdk-macros 0.14.0
3. Merge and publish ftl-sdk-macros 0.14.0
4. Create a normal PR to update ftl-sdk's dependency from `^0.13` to `^0.14`
5. This PR can include any needed code changes for the breaking change
6. When merged, release-please will create a release PR for ftl-sdk

### Scenario 3: Feature in ftl-sdk only
1. Feature lands in main
2. Release-please creates PR for ftl-sdk
3. Merge it
4. ftl-sdk-macros is unaffected

## Why This Works

1. **Simple** - No complex plugins or coordination
2. **Predictable** - Same as if packages were in different repos
3. **Flexible** - Packages can evolve at their own pace
4. **Standard** - Uses normal Cargo/crates.io versioning practices

## Configuration

```json
// release-please-config.json
{
  "separate-pull-requests": true,  // Each package gets its own PR
  "packages": {
    "sdk/rust": {
      "release-type": "rust",
      "component": "sdk-rust",
      "package-name": "ftl-sdk"
    },
    "sdk/rust-macros": {
      "release-type": "rust", 
      "component": "sdk-rust-macros",
      "package-name": "ftl-sdk-macros"
    }
  }
}
```

```toml
# sdk/rust/Cargo.toml
[dependencies]
ftl-sdk-macros = { version = "^0.13", optional = true }  # Normal semver range
```

## No Magic Required

- No linked-versions plugin
- No cargo-workspace plugin  
- No extra-files configuration
- No path dependencies
- Just normal, boring, simple package management