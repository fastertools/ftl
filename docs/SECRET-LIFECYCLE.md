# Secret Lifecycle in FTL

This document traces how secrets flow through the FTL system from definition to runtime.

## 1. Definition Phase (ftl.toml)

```toml
[variables]
api_key = { required = true }  # No value stored here!
```

**What's stored:** Just the variable name and the fact it's required  
**Security:** ✅ Safe to commit - no actual secrets

## 2. Local Development (Environment Variables)

```bash
export SPIN_VARIABLE_API_KEY="dev-key-12345"
spin up
```

**What happens:**
- Spin reads from your shell environment
- Variable injected into WASM runtime
- Only accessible to components that reference it

**Security:** ⚠️ Be careful with shell history and `.env` files

## 3. Deployment (CLI Arguments)

```bash
ftl box deploy --variable API_KEY="prod-key-xyz789"
```

**What happens:**
1. CLI sends variable to FTL API over HTTPS
2. FTL API passes to Fermyon Cloud
3. Fermyon stores encrypted in their infrastructure
4. Never written to disk in your project

**Security:** ✅ Transmitted securely, stored encrypted

## 4. Runtime (In Production)

When your WASM component calls:
```rust
let key = variables::get("api_key")?;
```

**What happens:**
1. Spin runtime intercepts the call
2. Retrieves value from secure storage
3. Injects into WASM memory space
4. Only your component can access it

**Security:** ✅ Isolated per component, not accessible to other apps

## 5. What Gets Transpiled

When `ftl.toml` becomes `spin.toml`:

**Input (ftl.toml):**
```toml
[variables]
stripe_key = { required = true }

[tools.payment.variables]
api_key = "{{ stripe_key }}"
```

**Output (spin.toml):**
```toml
[variables]
stripe_key = { required = true }

[component.payment.variables]
api_key = "{{ stripe_key }}"
```

**Note:** Still no actual secret values! Just the structure.

## Security Boundaries

### What CAN Access Your Secrets:
- ✅ Your component (only the variables it declares)
- ✅ The Spin runtime (to inject them)
- ✅ Fermyon's infrastructure (encrypted at rest)

### What CANNOT Access Your Secrets:
- ❌ Other components in your app (unless explicitly shared)
- ❌ Other apps on the platform
- ❌ Anyone with access to your source code
- ❌ Anyone who can see your `spin.toml`

## Common Misconceptions

### Myth 1: "Variables are environment variables"
**Reality:** Variables can be *provided* by environment variables locally, but in production they're stored securely by the platform.

### Myth 2: "Templates expose secrets"
**Reality:** `{{ api_key }}` is just a placeholder. The actual value is injected at runtime.

### Myth 3: "All components can see all variables"
**Reality:** Components only see variables they explicitly reference in their `[component.variables]` section.

## Example: Complete Secret Flow

1. **Developer defines need for secret:**
   ```toml
   [variables]
   database_password = { required = true }
   ```

2. **During development:**
   ```bash
   SPIN_VARIABLE_DATABASE_PASSWORD="dev-pass" spin up
   ```
   - Secret exists only in developer's environment

3. **CI/CD deployment:**
   ```yaml
   - name: Deploy to FTL
     env:
       DB_PASS: ${{ secrets.DATABASE_PASSWORD }}
     run: |
       ftl box deploy --variable DATABASE_PASSWORD="$DB_PASS"
   ```
   - Secret pulled from GitHub/GitLab secrets
   - Transmitted securely to FTL

4. **In production:**
   ```rust
   let db_pass = variables::get("database_password")?;
   let conn = Database::connect(&format!(
       "postgres://user:{}@host/db", 
       db_pass
   ));
   ```
   - Secret injected only when needed
   - Never logged or exposed

5. **Updating secrets:**
   ```bash
   ftl box deploy --variable DATABASE_PASSWORD="new-pass-456"
   ```
   - Old value replaced atomically
   - No need to modify code

## Best Practices Summary

1. **Define as required:** Always use `{ required = true }` for secrets
2. **Never commit values:** Only commit variable names, not values
3. **Use descriptive names:** `github_oauth_secret` not just `secret`
4. **Validate early:** Check variables exist at component startup
5. **Never log secrets:** Avoid exposing in logs or error messages
6. **Rotate regularly:** Redeploy with new values periodically

This architecture ensures secrets are never stored in your codebase while remaining easy to manage and deploy.