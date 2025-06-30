use ftl_sdk_rs::{ftl_mcp_gateway, gateway::{GatewayConfig, ToolEndpoint}, mcp::ServerInfo};

// Configure the gateway with the tools it should proxy to
fn create_gateway_config() -> GatewayConfig {
    GatewayConfig {
        tools: vec![
            ToolEndpoint {
                name: "weather",
                route: "/weather",
                description: Some("Get weather information for a location".to_string()),
            },
            ToolEndpoint {
                name: "calculator",
                route: "/calculator",
                description: Some("Perform mathematical calculations".to_string()),
            },
            ToolEndpoint {
                name: "translator",
                route: "/translator",
                description: Some("Translate text between languages".to_string()),
            },
        ],
        server_info: ServerInfo {
            name: "mcp-gateway".to_string(),
            version: "1.0.0".to_string(),
        },
        // Use Spin's local service chaining for internal requests
        base_url: "http://self".to_string(),
    }
}

// Create the gateway component
ftl_mcp_gateway!(create_gateway_config());