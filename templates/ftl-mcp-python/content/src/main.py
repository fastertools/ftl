"""
{{project-name | title_case}} - An FTL MCP tool written in Python.

This tool demonstrates how to create MCP tools using the FTL Python SDK.
"""

from ftl_sdk import create_tools, ToolResponse


def example_tool_handler(input_data):
    """
    Example tool handler that processes messages.
    
    Args:
        input_data: Dictionary containing the tool input
        
    Returns:
        ToolResponse with the processed result
    """
    message = input_data.get('message', '')
    # TODO: Implement your tool logic here
    return ToolResponse.text(f"Processed: {message}")


# Create the handler with your tools
Handler = create_tools({
    # Replace 'exampleTool' with your actual tool name
    "exampleTool": {
        "description": "An example tool that processes messages",
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
        "handler": example_tool_handler
    }
    
    # Add more tools here as needed:
    # "anotherTool": {
    #     "description": "Another tool description",
    #     "inputSchema": {
    #         "type": "object",
    #         "properties": {
    #             "param": {"type": "string"}
    #         }
    #     },
    #     "handler": another_handler
    # }
})