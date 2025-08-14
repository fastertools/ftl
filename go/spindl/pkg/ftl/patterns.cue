package ftl

// ===========================================================================
// Stage 1: Source formats → SpinDL Intermediate Model
// ===========================================================================

// Layer 3: FTL Application (source format - what users write)
#FTLApplication: {
	name!:        string & =~"^[a-z][a-z0-9-]*$"
	version?:     string | *"0.1.0"
	description?: string
	tools?: [...#Tool]
	access?:  "public" | "private" | *"public"
	auth?: #AuthConfig
}

#Tool: {
	id!: string & =~"^[a-z][a-z0-9-]*$"
	source!: #ToolSource
	build?: #BuildConfig
	environment?: [string]: string
}

#ToolSource: #LocalSource | #RegistrySource
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
// Layer 2: SpinDL Intermediate Model
// ===========================================================================
// This is the normalized, validated intermediate representation

#SpinDLApp: {
	name!:    string
	version!: string
	description?: string
	
	// All components with their configuration
	components!: [string]: #SpinComponent
	
	// HTTP routing configuration
	routes!: [...#Route]
}

#SpinComponent: {
	source!: string | {
		registry!: string
		package!:  string
		version!:  string
	}
	build?: {
		command!: string
		workdir?: string
		watch?: [...string]
	}
	environment?: [string]: string
	allowed_outbound_hosts?: [...string]
	variables?: [string]: string
	key_value_stores?: [...string]
}

#Route: {
	path!: string | {private: bool}
	component!: string
}

// ===========================================================================
// Transformation: FTL → SpinDL
// ===========================================================================

// Transform FTL application to SpinDL intermediate model
#FTLToSpinDL: {
	input: #FTLApplication
	
	output: #SpinDLApp & {
		name:    input.name
		version: input.version
		if input.description != _|_ {
			description: input.description
		}
		
		// Build components map
		components: {
			// User tools
			if input.tools != _|_ {
				for tool in input.tools {
					"\(tool.id)": {
						source: tool.source
						if tool.build != _|_ {
							build: tool.build
						}
						if tool.environment != _|_ {
							variables: tool.environment
						}
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
			}
			
			// MCP Authorizer (if auth enabled)
			if input.access != _|_ if input.access != "public" {
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
					if input.auth != _|_ {
						variables: {
							mcp_gateway_url: "http://ftl-mcp-gateway.spin.internal"
							if input.auth.jwt_issuer != _|_ {
								mcp_jwt_issuer: input.auth.jwt_issuer
							}
							if input.auth.jwt_audience != _|_ {
								mcp_jwt_audience: input.auth.jwt_audience
							}
							if input.auth.jwt_audience == _|_ {
								mcp_jwt_audience: input.name
							}
						}
					}
				}
			}
		}
		
		// Build routes
		routes: [
			// Entry point
			if input.access == _|_ || input.access == "public" {
				{path: "/...", component: "ftl-mcp-gateway"}
			},
			if input.access != _|_ if input.access != "public" {
				{path: "/...", component: "mcp-authorizer"}
			},
			
			// Private gateway route (if auth enabled)
			if input.access != _|_ if input.access != "public" {
				{path: {private: true}, component: "ftl-mcp-gateway"}
			},
			
			// Tool routes (always private)
			if input.tools != _|_ for tool in input.tools {
				{path: {private: true}, component: tool.id}
			},
		]
	}
}

// ===========================================================================
// Stage 2: SpinDL → Spin Manifest (spin.toml format)
// ===========================================================================

#SpinDLToManifest: {
	input: #SpinDLApp
	
	output: {
		spin_manifest_version: 2
		
		application: {
			name:    input.name
			version: input.version
			if input.description != _|_ {
				description: input.description
			}
		}
		
		component: input.components
		
		trigger: {
			http: [
				for r in input.routes {
					route:     r.path
					component: r.component
				}
			]
		}
	}
}