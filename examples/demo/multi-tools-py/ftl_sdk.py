"""
FTL SDK for Python - Zero-dependency SDK for building MCP tools.

This SDK provides a thin layer over Spin Python SDK to implement the
Model Context Protocol (MCP) for FTL tools.
"""

from spin_sdk import http
from spin_sdk.http import Request, Response
import json
from typing import Dict, Any, Callable, Optional, List, Union


# Type aliases for clarity
ToolHandler = Callable[[Dict[str, Any]], Dict[str, Any]]
JSONSchema = Dict[str, Any]


class ToolResponse:
    """Helper class for creating MCP-compliant tool responses."""
    
    @staticmethod
    def text(text: str) -> Dict[str, Any]:
        """Create a simple text response."""
        return {
            "content": [{
                "type": "text",
                "text": text
            }]
        }
    
    @staticmethod
    def error(error: str) -> Dict[str, Any]:
        """Create an error response."""
        return {
            "content": [{
                "type": "text",
                "text": error
            }],
            "isError": True
        }
    
    @staticmethod
    def with_structured(text: str, structured: Any) -> Dict[str, Any]:
        """Create a response with structured content."""
        return {
            "content": [{
                "type": "text",
                "text": text
            }],
            "structuredContent": structured
        }


class ToolContent:
    """Helper class for creating different types of content."""
    
    @staticmethod
    def text(text: str, annotations: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Create text content."""
        content = {"type": "text", "text": text}
        if annotations:
            content["annotations"] = annotations
        return content
    
    @staticmethod
    def image(data: str, mime_type: str, annotations: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Create image content."""
        content = {"type": "image", "data": data, "mimeType": mime_type}
        if annotations:
            content["annotations"] = annotations
        return content
    
    @staticmethod
    def audio(data: str, mime_type: str, annotations: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Create audio content."""
        content = {"type": "audio", "data": data, "mimeType": mime_type}
        if annotations:
            content["annotations"] = annotations
        return content
    
    @staticmethod
    def resource(resource: Dict[str, Any], annotations: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Create resource content."""
        content = {"type": "resource", "resource": resource}
        if annotations:
            content["annotations"] = annotations
        return content


def _camel_to_snake(name: str) -> str:
    """Convert camelCase to snake_case."""
    result = []
    for i, char in enumerate(name):
        if char.isupper() and i > 0:
            result.append('_')
        result.append(char.lower())
    return ''.join(result)


def create_tools(tools: Dict[str, Dict[str, Any]]) -> type:
    """
    Create a Spin HTTP handler for MCP tools.
    
    Args:
        tools: Dictionary mapping tool names to tool definitions.
               Each definition should have:
               - description: Tool description
               - inputSchema: JSON Schema for input validation
               - handler: Function that processes input and returns response
               - name (optional): Override the tool name
               - outputSchema (optional): JSON Schema for output
               - annotations (optional): Tool behavior hints
    
    Returns:
        A Spin IncomingHandler class that implements the MCP protocol.
    
    Example:
        Handler = create_tools({
            "echo": {
                "description": "Echo the input",
                "inputSchema": {
                    "type": "object",
                    "properties": {"message": {"type": "string"}},
                    "required": ["message"]
                },
                "handler": lambda input: ToolResponse.text(f"Echo: {input['message']}")
            }
        })
    """
    
    class IncomingHandler(http.IncomingHandler):
        def handle_request(self, request: Request) -> Response:
            path = request.uri
            method = request.method
            
            # Handle GET / - return tool metadata
            if method == "GET" and path == "/":
                metadata = []
                for key, tool in tools.items():
                    # Use explicit name if provided, otherwise convert from key
                    tool_name = tool.get("name", _camel_to_snake(key))
                    metadata.append({
                        "name": tool_name,
                        "title": tool.get("title"),
                        "description": tool.get("description", ""),
                        "inputSchema": tool.get("inputSchema", {"type": "object"}),
                        "outputSchema": tool.get("outputSchema"),
                        "annotations": tool.get("annotations"),
                        "_meta": tool.get("_meta")
                    })
                
                # Remove None values from metadata
                clean_metadata = []
                for item in metadata:
                    clean_item = {k: v for k, v in item.items() if v is not None}
                    clean_metadata.append(clean_item)
                
                return Response(
                    200,
                    {"content-type": "application/json"},
                    bytes(json.dumps(clean_metadata), "utf-8")
                )
            
            # Handle POST /{tool_name} - execute tool
            elif method == "POST":
                tool_name = path.lstrip('/')
                
                # Find the tool by name
                tool_entry = None
                for key, tool in tools.items():
                    effective_name = tool.get("name", _camel_to_snake(key))
                    if effective_name == tool_name:
                        tool_entry = tool
                        break
                
                if tool_entry is None:
                    error_response = ToolResponse.error(f"Tool '{tool_name}' not found")
                    return Response(
                        404,
                        {"content-type": "application/json"},
                        bytes(json.dumps(error_response), "utf-8")
                    )
                
                # Execute the tool
                try:
                    body = request.body.decode("utf-8") if request.body else "{}"
                    input_data = json.loads(body)
                    handler = tool_entry["handler"]
                    result = handler(input_data)
                    
                    return Response(
                        200,
                        {"content-type": "application/json"},
                        bytes(json.dumps(result), "utf-8")
                    )
                except Exception as e:
                    error_response = ToolResponse.error(f"Tool execution failed: {str(e)}")
                    return Response(
                        400,
                        {"content-type": "application/json"},
                        bytes(json.dumps(error_response), "utf-8")
                    )
            
            # Method not allowed
            return Response(
                405,
                {"content-type": "text/plain"},
                b"Method not allowed"
            )
    
    return IncomingHandler


# Type guards for content types
def is_text_content(content: Dict[str, Any]) -> bool:
    """Check if content is text type."""
    return content.get("type") == "text"


def is_image_content(content: Dict[str, Any]) -> bool:
    """Check if content is image type."""
    return content.get("type") == "image"


def is_audio_content(content: Dict[str, Any]) -> bool:
    """Check if content is audio type."""
    return content.get("type") == "audio"


def is_resource_content(content: Dict[str, Any]) -> bool:
    """Check if content is resource type."""
    return content.get("type") == "resource"