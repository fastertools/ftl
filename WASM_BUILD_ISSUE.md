# WASM Build Issue

## Problem
The WASM components fail to build locally due to ring dependency not supporting `wasm32-wasip1` target.

## Root Cause
1. `Cargo.lock` IS tracked but has diverged locally
2. WASM builds in CI only run when components/ files change
3. Recent releases haven't touched components, so CI hasn't tested WASM
4. Dependencies have drifted to versions that now require ring

## Evidence
- Commit df745fd passed CI but only changed Cargo.lock and cli/Cargo.toml
- WASM CI job has condition: `if: inputs.components-changed == 'true'`
- Last component code change was July 24 (commit aba9803)
- Both old and new Cargo.lock have jsonwebtoken v9.3.1 (but transitive deps differ)

## Why This Happened
This is a **drift regression** - the build was working, but as dependencies updated over time without component changes triggering WASM CI, the build broke undetected.

## Solutions

### Immediate
1. Revert to last known working Cargo.lock
2. Or fix component dependencies to use WASM-compatible alternatives

### Long-term
1. Always run WASM builds in CI (remove conditional)
2. For WASM components, switch to WASM-compatible JWT library:
   - `jwt-compact` or `jwt-simple` instead of `jsonwebtoken`
   - Remove unused `reqwest` dependency (component already uses Spin SDK for HTTP)
3. Pin exact versions in component Cargo.toml files