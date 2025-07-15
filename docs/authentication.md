# Authentication

FTL includes built-in authentication to secure your deployed tools and manage access.

## Login

To authenticate with FTL:

```bash
# Basic login (opens browser automatically)
ftl login

# Login without auto-opening browser
ftl login --no-browser
```

During login, you'll be shown:
- A verification URL to visit in your browser
- A user code to enter for authentication

## Credential Storage

FTL stores authentication credentials securely using your operating system's native credential storage:
- **macOS**: Keychain
- **Windows**: Windows Credential Manager  
- **Linux**: Secret Service API (GNOME Keyring, KWallet, etc.)

Stored credentials include:
- Access token for API authentication
- Refresh token for automatic token renewal
- Token expiration time
- AuthKit domain information

## Managing Authentication

```bash
# Check authentication status
ftl auth status

# Log out (removes stored credentials)
ftl logout
```

## Environment Variables

For CI/CD environments or custom deployments, you can configure authentication via environment variables:

```bash
# Set your AuthKit domain (if using a custom instance)
export FTL_AUTHKIT_DOMAIN=your-tenant.authkit.app

# Set your OAuth client ID (for custom Connect applications)
export FTL_CLIENT_ID=your_client_id

ftl login
```

## Security Notes

- Credentials are never stored in plain text
- Access tokens automatically refresh when expired
- Each login session is tied to your specific AuthKit domain
- Use `ftl logout` when switching between different FTL environments

## Technical Details

FTL uses the OAuth 2.0 Device Authorization Flow (RFC 8628) for authentication, which is optimized for command-line applications. This flow:
- Doesn't require a local web server
- Works on headless systems
- Provides a secure way to authenticate without embedding secrets

The authentication system integrates with WorkOS Connect, requiring a public OAuth application configured in your WorkOS dashboard.