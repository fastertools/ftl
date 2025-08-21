package mcp.authorization

import rego.v1

# Default deny
default allow := false

# Component-level authorization check
component_allowed if {
    # Admin role can access all components
    "admin" in input.token.claims.roles
}

component_allowed if {
    # If no component is specified (accessing root /mcp), check general access
    not input.request.component
    "mcp:access" in input.token.scopes
}

component_allowed if {
    # Check if user has specific component access
    input.request.component
    allowed_components := data.user_components[input.token.sub]
    input.request.component in allowed_components
}

component_allowed if {
    # Check role-based component access
    input.request.component
    component_roles := data.component_roles[input.request.component]
    some role in input.token.claims.roles
    role in component_roles
}

# Tool-level authorization (when MCP body inspection is enabled)
tool_allowed if {
    # If no MCP data in input, this is not a tool call - allow if component is allowed
    not input.mcp
    component_allowed
}

tool_allowed if {
    # Allow tool discovery for authorized components
    component_allowed
    input.mcp.method == "tools/list"
}

tool_allowed if {
    # Allow prompt and resource listing for authorized components
    component_allowed
    input.mcp.method in ["prompts/list", "resources/list"]
}

tool_allowed if {
    # Check tool-specific permissions
    component_allowed
    input.mcp.method == "tools/call"
    input.mcp.tool
    
    # Check if tool is in user's allowed tools for this component
    user_tools := data.user_tools[input.token.sub][input.request.component]
    input.mcp.tool in user_tools
}

tool_allowed if {
    # Check role-based tool permissions
    component_allowed
    input.mcp.method == "tools/call"
    input.mcp.tool
    
    # Get required role for this tool
    tool_config := data.tool_permissions[input.request.component][input.mcp.tool]
    tool_config.required_role in input.token.claims.roles
}

# Dangerous tool protection
dangerous_tool_denied if {
    input.mcp.method == "tools/call"
    input.mcp.tool in data.dangerous_tools
    not "super_admin" in input.token.claims.roles
}

# Rate limiting check (optional)
rate_limit_exceeded if {
    input.mcp.method == "tools/call"
    rate_limit := data.rate_limits[input.token.sub]
    # This would need external data about current usage
    # Placeholder for rate limiting logic
    false
}

# Final authorization decision
allow if {
    component_allowed
    tool_allowed
    not dangerous_tool_denied
    not rate_limit_exceeded
}

# Explicit deny reasons for better debugging
deny_reason["Component access denied"] if {
    not component_allowed
}

deny_reason["Tool access denied"] if {
    component_allowed
    not tool_allowed
}

deny_reason["Dangerous tool requires super_admin role"] if {
    dangerous_tool_denied
}

deny_reason["Rate limit exceeded"] if {
    rate_limit_exceeded
}