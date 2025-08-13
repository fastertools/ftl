// Advanced MCP Application using CUE
import "spinc.io/solutions/mcp"

// Environment-specific configuration
env: {
    OAUTH_ISSUER: string | *"https://auth.example.com"
    ENABLE_DEBUG: bool | *false
}

// Define the application
app: mcp.#McpApplication & {
    name: "advanced-mcp"
    version: "2.0.0"
    description: "Advanced MCP application with dynamic configuration"
    
    // Auth configuration from environment
    auth: {
        enabled: true
        issuer: env.OAUTH_ISSUER
        audience: ["api.example.com", "admin.example.com"]
        
        oauth: {
            authorize_endpoint: "\(env.OAUTH_ISSUER)/authorize"
            token_endpoint: "\(env.OAUTH_ISSUER)/token"
            userinfo_endpoint: "\(env.OAUTH_ISSUER)/userinfo"
        }
    }
    
    // Component versions
    mcp: {
        gateway: "ghcr.io/fastertools/mcp-gateway:0.0.11"
        authorizer: "ghcr.io/fastertools/mcp-authorizer:0.0.14"
        validate_arguments: true
    }
    
    // Dynamic component configuration
    components: {
        // Always include core tools
        calculator: {
            source: "ghcr.io/example/calculator:2.0.0"
            route: "/calc/..."
            allowed_outbound_hosts: ["https://api.mathjs.org"]
        }
        
        weather: {
            source: "ghcr.io/example/weather:1.5.0"
            route: "/weather/..."
            allowed_outbound_hosts: ["https://api.openweathermap.org"]
            environment: {
                API_KEY: "{{ weather_api_key }}"
            }
        }
        
        // Conditionally include debug tools
        if env.ENABLE_DEBUG {
            debugger: {
                source: "ghcr.io/example/debug-tools:latest"
                route: "/debug/..."
                allowed_outbound_hosts: ["http://localhost:*"]
            }
        }
    }
    
    // Application variables
    variables: {
        weather_api_key: { required: true }
        log_level: { default: "info" }
        if env.ENABLE_DEBUG {
            debug_port: { default: "9229" }
        }
    }
}