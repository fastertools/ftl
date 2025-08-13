// MCP Application L3 Construct
package solutions

import (
    "spinc.io/core"
    "strings"
)

// High-level MCP application configuration
#McpApplication: {
    // Required fields
    name!: string & =~"^[a-zA-Z][a-zA-Z0-9_-]*$"
    
    // Optional fields with defaults
    version?: string | *"0.1.0"
    description?: string | *"MCP application"
    authors?: [...string] | *[]
    
    // Auth configuration
    auth?: #AuthConfig | *{ enabled: false }
    
    // MCP component versions
    mcp?: {
        gateway?: string | *"ghcr.io/fastertools/mcp-gateway:latest"
        authorizer?: string | *"ghcr.io/fastertools/mcp-authorizer:latest"
        validate_arguments?: bool | *false
    }
    
    // User components
    components?: [string]: #UserComponent
    
    // Application variables
    variables?: [string]: string | { default: string } | { required: true }
    
    // Computed fields
    _authEnabled: bool
    if auth != _|_ {
        _authEnabled: auth.enabled | *false
    }
    if auth == _|_ {
        _authEnabled: false
    }
    
    // Synthesized Spin manifest
    manifest: core.#SpinManifest & {
        spin_manifest_version: 2
        
        application: {
            name: name
            if version != _|_ {
                version: version
            }
            if description != _|_ {
                description: description
            }
            if len(authors) > 0 {
                authors: authors
            }
        }
        
        // Build variables
        variables: _buildVariables
        
        // Build components based on auth state
        component: _buildComponents
        
        // Build triggers
        trigger: http: _buildHttpTriggers
    }
    
    // Helper: Build variables
    _buildVariables: [string]: core.#Variable
    _buildVariables: {
        // User variables
        if variables != _|_ {
            for k, v in variables {
                if (v & string) != _|_ {
                    "\(k)": default: v
                }
                if (v & {default: string}) != _|_ {
                    "\(k)": v
                }
                if (v & {required: true}) != _|_ {
                    "\(k)": v
                }
            }
        }
        
        // System variables
        "auth_enabled": default: "\(_authEnabled)"
        
        if components != _|_ {
            "component_names": default: strings.Join([for k, _ in components {k}], ",")
        }
        if components == _|_ {
            "component_names": default: ""
        }
        
        // MCP variables
        "mcp_validate_arguments": default: "\(mcp.validate_arguments)"
        
        // Auth-related variables
        if _authEnabled && auth != _|_ {
            "mcp_provider_type": default: "jwt"
            
            if auth.issuer != _|_ {
                "mcp_jwt_issuer": default: auth.issuer
            }
            
            if auth.audience != _|_ {
                "mcp_jwt_audience": default: strings.Join(auth.audience, ",")
            }
            
            if auth.jwks_uri != _|_ {
                "mcp_jwt_jwks_uri": default: auth.jwks_uri
            }
            
            if auth.public_key != _|_ {
                "mcp_jwt_public_key": default: auth.public_key
            }
            
            if auth.algorithm != _|_ {
                "mcp_jwt_algorithm": default: auth.algorithm
            }
            
            if auth.required_scopes != _|_ {
                "mcp_jwt_required_scopes": default: auth.required_scopes
            }
            
            if auth.oauth != _|_ {
                if auth.oauth.authorize_endpoint != _|_ {
                    "mcp_oauth_authorize_endpoint": default: auth.oauth.authorize_endpoint
                }
                if auth.oauth.token_endpoint != _|_ {
                    "mcp_oauth_token_endpoint": default: auth.oauth.token_endpoint
                }
                if auth.oauth.userinfo_endpoint != _|_ {
                    "mcp_oauth_userinfo_endpoint": default: auth.oauth.userinfo_endpoint
                }
            }
            
            if auth.allowed_subjects != _|_ {
                "mcp_allowed_subjects": default: strings.Join(auth.allowed_subjects, ",")
            }
        }
        
        if !_authEnabled {
            // Empty auth variables when disabled
            "mcp_provider_type": default: ""
            "mcp_jwt_issuer": default: ""
            "mcp_jwt_audience": default: ""
            "mcp_jwt_jwks_uri": default: ""
            "mcp_jwt_public_key": default: ""
            "mcp_jwt_algorithm": default: ""
            "mcp_jwt_required_scopes": default: ""
            "mcp_oauth_authorize_endpoint": default: ""
            "mcp_oauth_token_endpoint": default: ""
            "mcp_oauth_userinfo_endpoint": default: ""
            "mcp_allowed_subjects": default: ""
        }
    }
    
    // Helper: Build components
    _buildComponents: [string]: core.#Component
    _buildComponents: {
        // MCP components
        if _authEnabled {
            // With auth: authorizer is "mcp", gateway is "ftl-mcp-gateway"
            "mcp": _buildMcpComponent(mcp.authorizer, true)
            "ftl-mcp-gateway": _buildMcpComponent(mcp.gateway, false)
        }
        if !_authEnabled {
            // Without auth: gateway is "mcp"
            "mcp": _buildMcpComponent(mcp.gateway, false)
        }
        
        // User components
        if components != _|_ {
            for name, config in components {
                "\(name)": _buildUserComponent(config)
            }
        }
    }
    
    // Helper: Build HTTP triggers
    _buildHttpTriggers: [...core.#HttpTrigger]
    _buildHttpTriggers: [
        // MCP route (always /mcp/...)
        {
            route: "/mcp/..."
            component: "mcp"
        },
        
        // If auth enabled, add explicit authorizer route
        if _authEnabled {
            {
                route: "/auth/..."
                component: "mcp"
            }
        },
        
        // User component routes (if any)
        if components != _|_ {
            for name, config in components {
                if config.route != _|_ {
                    {
                        route: config.route
                        component: name
                    }
                }
            }
        },
    ]
}

// Auth configuration
#AuthConfig: {
    enabled: bool | *true
    
    // JWT configuration
    issuer?: string
    audience?: [...string] & len(audience) > 0
    jwks_uri?: string
    public_key?: string
    algorithm?: string
    required_scopes?: string
    
    // OAuth endpoints
    oauth?: {
        authorize_endpoint?: string
        token_endpoint?: string
        userinfo_endpoint?: string
    }
    
    // Access control
    allowed_subjects?: [...string]
}

// User component configuration
#UserComponent: {
    // Source (required)
    source!: string  // Can be file path or registry reference
    
    // Optional routing
    route?: string
    
    // Optional configuration
    allowed_outbound_hosts?: [...string]
    environment?: [string]: string
    variables?: [string]: string
    files?: [...string | core.#FileMount]
    
    // Build configuration (for local components)
    build?: core.#BuildConfig
}

// Helper function to build MCP component
_buildMcpComponent: {
    input: string
    isAuthorizer: bool
    
    output: core.#Component
    output: {
        let normalized = (core.#NormalizeSource & { "input": input }).output
        source: normalized
        
        // MCP components need specific permissions
        allowed_outbound_hosts: [
            "http://127.0.0.1:*",
            "http://localhost:*",
        ]
        
        // Component-specific variables
        variables: {
            if isAuthorizer {
                // Authorizer gets all auth-related variables
                "provider_type": "{{ mcp_provider_type }}"
                "jwt_issuer": "{{ mcp_jwt_issuer }}"
                "jwt_audience": "{{ mcp_jwt_audience }}"
                "jwt_jwks_uri": "{{ mcp_jwt_jwks_uri }}"
                "jwt_public_key": "{{ mcp_jwt_public_key }}"
                "jwt_algorithm": "{{ mcp_jwt_algorithm }}"
                "jwt_required_scopes": "{{ mcp_jwt_required_scopes }}"
                "oauth_authorize_endpoint": "{{ mcp_oauth_authorize_endpoint }}"
                "oauth_token_endpoint": "{{ mcp_oauth_token_endpoint }}"
                "oauth_userinfo_endpoint": "{{ mcp_oauth_userinfo_endpoint }}"
                "allowed_subjects": "{{ mcp_allowed_subjects }}"
            }
            if !isAuthorizer {
                // Gateway gets validate arguments flag
                "validate_arguments": "{{ mcp_validate_arguments }}"
            }
        }
    }
}

// Helper function to build user component
_buildUserComponent: {
    input: #UserComponent
    
    output: core.#Component
    output: {
        let normalized = (core.#NormalizeSource & { "input": input.source }).output
        source: normalized
        
        if input.allowed_outbound_hosts != _|_ {
            allowed_outbound_hosts: input.allowed_outbound_hosts
        }
        
        if input.environment != _|_ {
            environment: input.environment
        }
        
        if input.variables != _|_ {
            variables: input.variables
        }
        
        if input.files != _|_ {
            files: input.files
        }
        
        if input.build != _|_ {
            build: input.build
        }
    }
}