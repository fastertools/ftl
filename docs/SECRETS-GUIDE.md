# Secrets Management in FTL

This guide explains how to handle secrets and sensitive configuration in FTL projects using the variable system.

## How Secrets Work in FTL

FTL leverages Spin's variable system for managing secrets. When you deploy to FTL Engine (which uses Fermyon's platform under the hood), secrets are handled through deployment-time variables.

### Key Concepts

1. **No Secrets in Code**: Secrets should never be committed to your repository
2. **Runtime Injection**: Secrets are provided at deployment time via `--variable` flags
3. **Secure Storage**: Once deployed, variables are securely stored by the platform
4. **Component Isolation**: Only components that explicitly reference variables can access them

## Defining Secrets in ftl.toml

Mark sensitive variables as required in your `ftl.toml`:

```toml
[variables]
# Secrets - must be provided at deployment
api_key = { required = true }
database_password = { required = true }
jwt_secret = { required = true }

# Non-sensitive config with defaults
api_endpoint = { default = "https://api.example.com" }
log_level = { default = "info" }
```

## Using Secrets in Components

Components access secrets through variable references:

```toml
[tools.api-client]
path = "api-client"

[tools.api-client.variables]
# Reference the required secrets
auth_token = "{{ api_key }}"
db_password = "{{ database_password }}"
signing_key = "{{ jwt_secret }}"

# Reference config with defaults
endpoint = "{{ api_endpoint }}"
```

## Providing Secrets

### During Development (Local)

When running locally with `spin up`, use environment variables:

```bash
# Set secrets via environment
export SPIN_VARIABLE_API_KEY="dev-key-12345"
export SPIN_VARIABLE_DATABASE_PASSWORD="dev-password"
export SPIN_VARIABLE_JWT_SECRET="dev-secret"

# Run the application
spin up
```

Or provide them inline:

```bash
SPIN_VARIABLE_API_KEY="dev-key-12345" \
SPIN_VARIABLE_DATABASE_PASSWORD="dev-password" \
SPIN_VARIABLE_JWT_SECRET="dev-secret" \
spin up
```

### During Deployment

When deploying to FTL Engine, provide secrets via `--variable` flags:

```bash
ftl eng deploy \
  --variable API_KEY="production-key-xyz789" \
  --variable DATABASE_PASSWORD="prod-db-password" \
  --variable JWT_SECRET="prod-signing-secret"
```

## Important Security Notes

### What Happens to Your Secrets

1. **During Deploy**: Secrets are transmitted securely to the FTL platform
2. **Storage**: The platform stores variables securely (similar to how Heroku/Vercel handle env vars)
3. **Runtime**: Variables are injected into your WebAssembly components at runtime
4. **Isolation**: Each component only has access to the variables it explicitly references

### What NOT to Do

❌ **Never commit secrets to git**:
```toml
# BAD - Never do this!
[tools.api.variables]
api_key = "sk_live_abcd1234"  # NEVER hardcode secrets
```

❌ **Never log secrets**:
```rust
// BAD - Never log sensitive values
let api_key = variables::get("api_key")?;
println!("Using API key: {}", api_key);  // DON'T DO THIS
```

❌ **Never expose secrets in errors**:
```rust
// BAD - Don't include secret values in error messages
return Err(format!("Failed to connect with password: {}", password));
```

### Best Practices

✅ **Use required variables for all secrets**:
```toml
[variables]
stripe_key = { required = true }  # Good - no default for secrets
```

✅ **Use descriptive variable names**:
```toml
[variables]
github_oauth_client_secret = { required = true }  # Clear what this is
database_connection_string = { required = true }  # Self-documenting
```

✅ **Document required variables**:
```toml
# Create a .env.example for development
# (Don't commit the actual .env file!)
API_KEY=your-api-key-here
DATABASE_PASSWORD=your-password-here
JWT_SECRET=your-secret-here
```

✅ **Validate secrets exist before using**:
```rust
// Good - Check if variable exists and handle errors
let api_key = match variables::get("api_key") {
    Ok(key) => key,
    Err(_) => return Err("API_KEY not configured".into()),
};
```

## Updating Secrets

To update a secret after deployment, simply redeploy with the new value:

```bash
# Deploy with updated secret
ftl eng deploy --variable API_KEY="new-production-key-abc123"
```

The platform will update the variable and restart your application with the new value.

## Local Development Workflow

1. Create a `.env` file for local development (git-ignored):
   ```bash
   # .env (DO NOT COMMIT)
   API_KEY=dev-key-12345
   DATABASE_PASSWORD=local-dev-password
   ```

2. Add `.env` to `.gitignore`:
   ```
   .env
   .env.local
   ```

3. Load variables when running locally:
   ```bash
   # Using a tool like direnv or manually:
   source .env
   
   # Then run with Spin
   spin up
   ```

## Example: API Integration

Here's a complete example of using secrets for an API integration:

```toml
# ftl.toml
[project]
name = "weather-toolbox"

[variables]
# Required secrets
openweather_api_key = { required = true }

# Configuration with defaults
weather_api_url = { default = "https://api.openweathermap.org/data/2.5" }
cache_ttl_seconds = { default = "300" }

[tools.weather]
path = "weather"
allowed_outbound_hosts = ["{{ weather_api_url }}"]

[tools.weather.variables]
api_key = "{{ openweather_api_key }}"
base_url = "{{ weather_api_url }}"
cache_ttl = "{{ cache_ttl_seconds }}"
```

Deploy with:
```bash
ftl eng deploy --variable OPENWEATHER_API_KEY="your-actual-api-key"
```

## Limitations

Currently, FTL (via Spin on Fermyon) supports:
- ✅ Deployment-time variables via `--variable`
- ✅ Environment variables for local development
- ✅ Secure storage of variables on the platform

Not yet supported on Fermyon:
- ❌ HashiCorp Vault integration
- ❌ Azure Key Vault integration  
- ❌ Dynamic secret rotation without redeployment

The FTL team is working on enhanced secret management solutions for future releases.

## Summary

The current secret management in FTL:
1. Define secrets as `required = true` in `[variables]`
2. Reference them in components with `{{ secret_name }}`
3. Provide values via `--variable` during deployment
4. Platform securely stores and injects them at runtime

This approach ensures secrets are never stored in code while remaining simple to use and deploy.