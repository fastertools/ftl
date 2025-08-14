// FTL L3 Patterns - High-level MCP orchestration constructs
package ftl

import (
    "strings"
    core "github.com/fastertools/spindl/core"
)

// FTLApplication is the top-level L3 construct for MCP apps
#FTLApplication: {
    // User-facing simple schema
    application: {
        name!: string
        version?: string | *"0.1.0"
        description?: string
    }
    
    // MCP tools (user components)
    tools?: [...#Tool]
    
    // Access control (simplified)
    access?: "public" | "private" | *"public"
    
    auth?: {
        provider!: "workos" | "auth0" | "custom"
        org_id?: string
        jwt_issuer?: string
        jwt_audience?: string
    }
    
    // Synthesize to L2 SpinDL format
    _synthesized: core.#SpinManifest & {
        spin_manifest_version: 2
        
        application: {
            name: application.name
            version: application.version
            description: application.description
        }
        
        // Generate all components
        component: {
            // User tools become private components
            for tool in tools {
                "\(tool.id)": {
                    source: tool.source
                    if tool.build != _|_ {
                        build: tool.build
                    }
                }
            }
            
            // Always add MCP gateway
            "ftl-mcp-gateway": {
                source: {
                    registry: "ghcr.io"
                    package: "fastertools:mcp-gateway"
                    version: "0.0.13-alpha.0"
                }
                allowed_outbound_hosts: ["http://*.spin.internal"]
                variables: {
                    component_names: _toolNames
                }
            }
            
            // Add authorizer if not public
            if access != "public" {
                "mcp-authorizer": {
                    source: {
                        registry: "ghcr.io"
                        package: "fastertools:mcp-authorizer"
                        version: "0.0.15-alpha.0"
                    }
                    allowed_outbound_hosts: [
                        "http://*.spin.internal",
                        "https://*.authkit.app",
                        "https://*.workos.com"
                    ]
                    if auth != _|_ {
                        variables: {
                            if auth.jwt_issuer != _|_ {
                                mcp_jwt_issuer: auth.jwt_issuer
                            }
                            if auth.jwt_audience != _|_ {
                                mcp_jwt_audience: auth.jwt_audience
                            }
                            mcp_gateway_url: "http://ftl-mcp-gateway.spin.internal"
                        }
                    }
                }
            }
        }
        
        // Generate triggers automatically
        trigger: {
            http: [
                // Public entry point
                if access == "public" {
                    {
                        route: "/..."
                        component: "ftl-mcp-gateway"
                    }
                },
                if access != "public" {
                    {
                        route: "/..."
                        component: "mcp-authorizer"
                    },
                    {
                        route: { private: true }
                        component: "ftl-mcp-gateway"  
                    }
                },
                // All user tools are private
                for tool in tools {
                    {
                        route: { private: true }
                        component: tool.id
                    }
                }
            ]
        }
    }
    
    // Helper to collect tool names
    _toolNames: strings.Join([for t in *tools | [] { t.id }], ",")
}

// Tool represents an MCP tool component
#Tool: {
    id!: string
    source!: string | core.#URLSource | core.#RegistrySource
    build?: core.#BuildConfig
    environment?: [string]: string
}

// Example usage:
example: #FTLApplication & {
    application: {
        name: "my-mcp-app"
    }
    
    tools: [
        {
            id: "calculator"
            source: "./calc.wasm"
        },
        {
            id: "weather"
            source: {
                registry: "ghcr.io"
                package: "example/weather"
                version: "1.0.0"
            }
        }
    ]
    
    access: "private"
    auth: {
        provider: "workos"
        org_id: "org_123"
    }
}

// The synthesized output is in example._synthesized