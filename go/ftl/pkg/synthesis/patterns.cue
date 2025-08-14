package ftl

import "strings"

// ===========================================================================
// FTL Application Types (what users write)
// ===========================================================================

#FTLApplication: {
	name!:        string & =~"^[a-z][a-z0-9-]*$"
	version:      string | *"0.1.0"
	description:  string | *""
	components:   [...#Component] | *[]
	access:       "public" | "private" | *"public"
	auth:         #AuthConfig | *{provider: "workos", org_id: "", jwt_issuer: "https://api.workos.com", jwt_audience: ""}
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
	provider!: "workos" | "custom"
	
	if provider == "workos" {
		org_id!: string
		jwt_issuer: *"https://api.workos.com" | string
		jwt_audience?: string
	}
	
	if provider == "custom" {
		jwt_issuer!: string
		jwt_audience!: string
	}
}

// ===========================================================================
// Direct Transformation: FTL → Spin Manifest
// ===========================================================================

#TransformToSpin: {
	input: #FTLApplication
	
	// Helper to determine if we need auth
	_needsAuth: input.access == "private"
	
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
			for comp in input.components {
				"\(comp.id)": {
					source: comp.source
					// Only include build for local sources (string type)
					if (comp.source & string) != _|_ && comp.build != _|_ {
						build: comp.build  
					}
					if comp.variables != _|_ {
						variables: comp.variables
					}
				}
			}
			
			// MCP Gateway (always present)
			"ftl-mcp-gateway": {
				source: {
					registry: "ghcr.io"
					package:  "fastertools:mcp-gateway"
					version:  "0.0.13-alpha.0"
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
						version:  "0.0.15-alpha.0"
					}
					allowed_outbound_hosts: [
						"http://*.spin.internal",
						"https://*.authkit.app",
						"https://*.workos.com",
					]
					variables: {
						mcp_gateway_url: "http://ftl-mcp-gateway.spin.internal"
						mcp_jwt_issuer: input.auth.jwt_issuer | *"https://api.workos.com"
						mcp_jwt_audience: input.auth.jwt_audience | *input.name
					}
				}
			}
		}
		
		// Build trigger configuration - using concatenation instead of conditionals in lists
		trigger: {
			// Base routes
			_publicRoutes: [{
				route: "/..."
				component: "ftl-mcp-gateway"
			}]
			
			_privateRoutes: [
				{
					route: "/..."
					component: "mcp-authorizer"
				},
				{
					route: {private: true}
					component: "ftl-mcp-gateway"
				}
			]
			
			// Component routes
			_componentRoutes: [for comp in input.components {
				route: {private: true}
				component: comp.id
			}]
			
			// Select routes based on access mode
			if _needsAuth {
				http: _privateRoutes + _componentRoutes
			}
			if !_needsAuth {
				http: _publicRoutes + _componentRoutes
			}
		}
	}
}