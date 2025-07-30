from ftl_sdk import create_tools, ToolResponse

# Create the echo tool handler
IncomingHandler = create_tools({
    "echo_py": {
        "description": "An MCP tool written in Python",
        "inputSchema": {
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The input message to process"
                }
            },
            "required": ["message"]
        },
        "handler": lambda input: ToolResponse.text(f"Processed: {input['message']}")
    }
})