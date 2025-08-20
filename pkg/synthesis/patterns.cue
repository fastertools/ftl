package ftl

import (
	"strings"
	"list"
)

// ===========================================================================
// FTL Application Types (what users write)
// ===========================================================================

#FTLApplication: {
	name!:        string & =~"^[a-z][a-z0-9-]*$"
	version:      string | *"0.1.0"
	description:  string | *""
	components:   [...#Component] | *[]
	// Access modes:
	// - public: No authentication required
	// - private: FTL auth, user-only access
	// - org: FTL auth, org-level access (or M2M tokens scoped to org)
	// - custom: User provides all auth configuration
	access:       "public" | "private" | "org" | "custom" | *"public"
	auth?:        #AuthConfig  // Required only for "custom" access
	// For org access mode - optional role filter
	allowed_roles?: [...string]
	// For org access mode - list of allowed user subjects
	allowed_subjects?: [...string]
}

#Component: {
	id!: string & =~"^[a-z][a-z0-9-]*$"
	source!: #ComponentSource
	build: #BuildConfig | *{command: "", workdir: "", watch: []}
	variables?: {[string]: string}
}

#ComponentSource: #LocalSource | #RegistrySource
#LocalSource: string
#RegistrySource: {
	registry!: string
	package!:  string
	version!:  string
}

#BuildConfig: {
	command!: string
	workdir?: string
	watch?: [...string]
}

#AuthConfig: {
	// For custom access mode - provide your JWT configuration
	jwt_issuer!: string
	jwt_audience!: string
	// Optional: Required scopes for token validation
	jwt_required_scopes?: [...string]
}

// ===========================================================================
// Input Transformation: Raw Input → FTL Application → Spin Manifest
// ===========================================================================

// Transform raw input data to a validated FTL application
#PlatformConfig: {
	gateway_version:     string | *"0.0.13-alpha.0"
	authorizer_version:  string | *"0.0.15-alpha.0"
}

#InputTransform: {
	input: _
	platform: #PlatformConfig  // Always required, but has defaults
	
	// Build validated FTL app from raw input
	app: #FTLApplication & {
		name:        input.name
		version:     input.version | *"0.1.0"
		description: input.description | *""
		
		// Components with defaults
		if input.components != _|_ {
			components: input.components
		}
		if input.components == _|_ {
			components: []
		}
		
		// Access mode with default
		if input.access != _|_ {
			access: input.access
		}
		if input.access == _|_ {
			access: "public"
		}
		
		// Pass through auth if present
		if input.auth != _|_ {
			auth: input.auth
		}
		
		// Pass through allowed_subjects if present
		if input.allowed_subjects != _|_ {
			allowed_subjects: input.allowed_subjects
		}
	}
	
	// Transform to Spin manifest
	manifest: (#TransformToSpin & {
		input: app
		platform: platform
	}).output
}

// ===========================================================================
// Direct Transformation: FTL → Spin Manifest
// ===========================================================================

#TransformToSpin: {
	input: #FTLApplication
	platform: #PlatformConfig  // Always required, has defaults
	
	// Helper to determine if we need auth
	_needsAuth: input.access == "private" || input.access == "org" || input.access == "custom"
	
	// Store platform versions in local fields for reference
	_gatewayVersion: platform.gateway_version
	_authorizerVersion: platform.authorizer_version
	
	output: {
		spin_manifest_version: 2
		
		application: {
			name:    input.name
			version: input.version
			if input.description != "" {
				description: input.description
			}
		}
		
		// Build components map
		component: {
			// User components
			// IMPORTANT: User components are intentionally restricted from accessing:
			// - key_value_stores: KV access is only granted to platform components
			// - sqlite_databases: Database access is not exposed to users
			// - ai_models: AI model access is not exposed to users
			// This ensures proper isolation and prevents resource abuse.
			// Only the following fields are copied from user configuration:
			for comp in input.components {
				"\(comp.id)": {
					source: comp.source
					// Only include build for local sources (string type)
					if (comp.source & string) != _|_ {
						if comp.build.command != "" {
							build: comp.build  
						}
					}
					if comp.variables != _|_ {
						variables: comp.variables
					}
					// NOTE: No key_value_stores, sqlite_databases, or ai_models
				}
			}
			
			// MCP Gateway (always present)
			"mcp-gateway": {
				source: {
					registry: "ghcr.io"
					package:  "fastertools:mcp-gateway"
					version: _gatewayVersion
				}
				allowed_outbound_hosts: ["http://*.spin.internal"]
				// Add component_names if there are user components
				if len(input.components) > 0 {
					variables: {
						component_names: strings.Join([for c in input.components {c.id}], ",")
					}
				}
			}
			
			// MCP Authorizer (added when auth is enabled using comprehension)
			// This produces either 0 or 1 component based on _needsAuth
			if _needsAuth {
				"mcp-authorizer": {
					source: {
						registry: "ghcr.io"
						package:  "fastertools:mcp-authorizer"
						version: _authorizerVersion
					}
					allowed_outbound_hosts: [
						"http://*.spin.internal",
						"https://*.authkit.app",
						"https://*.workos.com",
					]
					key_value_stores: ["default"]
					variables: {
						mcp_gateway_url: "http://mcp-gateway.spin.internal"
						
						// JWT configuration based on access mode
						if input.access == "private" {
							// For "private" mode - use FTL platform auth
							mcp_jwt_issuer: "https://divine-lion-50-staging.authkit.app"
							mcp_jwt_audience: "client_01JZM53FW3WYV08AFC4QWQ3BNB"
							mcp_jwt_jwks_uri: "https://divine-lion-50-staging.authkit.app/oauth2/jwks"
							
							// For private mode, inject allowed subjects if provided
							if input.allowed_subjects != _|_ {
								mcp_auth_allowed_subjects: strings.Join(input.allowed_subjects, ",")
							}
						}
						
						if input.access == "org" {
							// For "org" mode - use FTL platform auth
							mcp_jwt_issuer: "https://divine-lion-50-staging.authkit.app"
							mcp_jwt_audience: "client_01JZM53FW3WYV08AFC4QWQ3BNB"
							mcp_jwt_jwks_uri: "https://divine-lion-50-staging.authkit.app/oauth2/jwks"
							
							// For org mode, inject allowed subjects if provided
							if input.allowed_subjects != _|_ {
								mcp_auth_allowed_subjects: strings.Join(input.allowed_subjects, ",")
							}
						}
						
						if input.access == "custom" {
							// For "custom" mode - user must provide auth config
							mcp_jwt_issuer: input.auth.jwt_issuer | *"https://divine-lion-50-staging.authkit.app"
							mcp_jwt_audience: input.auth.jwt_audience | *input.name
						}
					}
				}
			}
		}
		
		// Build trigger configuration - using concatenation instead of conditionals in lists
		trigger: {
			// Base routes
			_publicRoutes: [{
				route: "/..."
				component: "mcp-gateway"
			}]
			
			_privateRoutes: [
				{
					route: "/..."
					component: "mcp-authorizer"
				},
				{
					route: {private: true}
					component: "mcp-gateway"
				}
			]
			
			// Component routes
			_componentRoutes: [for comp in input.components {
				route: {private: true}
				component: comp.id
			}]
			
			// Select routes based on access mode
			if _needsAuth {
				http: list.Concat([_privateRoutes, _componentRoutes])
			}
			if !_needsAuth {
				http: list.Concat([_publicRoutes, _componentRoutes])
			}
		}
	}
}