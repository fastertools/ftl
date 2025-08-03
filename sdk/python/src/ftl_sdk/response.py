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


class ToolResult:
    """
    Enhanced tool result builder with fluent API for FastMCP-style responses.
    
    Provides a chainable interface for building rich MCP responses with multiple
    content types, structured data, and proper error handling.
    
    Example:
        # Simple text response
        return ToolResult().text("Hello world")
        
        # Rich response with multiple content types
        return (ToolResult()
                .text("Process completed")
                .add_text("Details: Processing finished successfully")
                .with_structured({"status": "success", "count": 42})
                .with_progress(90))
        
        # Error response
        return ToolResult().error("Something went wrong", details={"code": 500})
    """
    
    def __init__(self):
        """Initialize a new ToolResult builder."""
        self._content: List[Dict[str, Any]] = []
        self._structured_content: Optional[Any] = None
        self._is_error: bool = False
        self._progress: Optional[float] = None
        self._meta: Optional[Dict[str, Any]] = None
    
    def text(self, text: str, annotations: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Add text content to the response.
        
        Args:
            text: The text content
            annotations: Optional annotations for the content
            
        Returns:
            Self for chaining
        """
        content = ToolContent.text(text, annotations)
        self._content.append(content)
        return self
    
    def add_text(self, text: str, annotations: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Add additional text content (alias for text() for clarity in chaining).
        
        Args:
            text: The text content
            annotations: Optional annotations for the content
            
        Returns:
            Self for chaining
        """
        return self.text(text, annotations)
    
    def image(self, data: str, mime_type: str, annotations: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Add image content to the response.
        
        Args:
            data: Base64-encoded image data
            mime_type: MIME type of the image
            annotations: Optional annotations for the content
            
        Returns:
            Self for chaining
        """
        content = ToolContent.image(data, mime_type, annotations)
        self._content.append(content)
        return self
    
    def audio(self, data: str, mime_type: str, annotations: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Add audio content to the response.
        
        Args:
            data: Base64-encoded audio data
            mime_type: MIME type of the audio
            annotations: Optional annotations for the content
            
        Returns:
            Self for chaining
        """
        content = ToolContent.audio(data, mime_type, annotations)
        self._content.append(content)
        return self
    
    def resource(self, resource: Dict[str, Any], annotations: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Add resource content to the response.
        
        Args:
            resource: Resource definition
            annotations: Optional annotations for the content
            
        Returns:
            Self for chaining
        """
        content = ToolContent.resource(resource, annotations)
        self._content.append(content)
        return self
    
    def with_structured(self, structured: Any) -> 'ToolResult':
        """
        Add structured content to the response.
        
        Args:
            structured: Any JSON-serializable structured data
            
        Returns:
            Self for chaining
        """
        self._structured_content = structured
        return self
    
    def with_progress(self, progress: float) -> 'ToolResult':
        """
        Add progress information to the response.
        
        Args:
            progress: Progress value (0.0 to 100.0)
            
        Returns:
            Self for chaining
        """
        self._progress = max(0.0, min(100.0, progress))
        return self
    
    def with_meta(self, meta: Dict[str, Any]) -> 'ToolResult':
        """
        Add metadata to the response.
        
        Args:
            meta: Metadata dictionary
            
        Returns:
            Self for chaining
        """
        if self._meta is None:
            self._meta = {}
        self._meta.update(meta)
        return self
    
    def error(self, message: str, details: Optional[Dict[str, Any]] = None) -> 'ToolResult':
        """
        Mark this result as an error.
        
        Args:
            message: Error message
            details: Optional error details
            
        Returns:
            Self for chaining
        """
        self._is_error = True
        self._content.append(ToolContent.text(message))
        
        if details:
            self.with_structured({"error_details": details})
        
        return self
    
    def build(self) -> Dict[str, Any]:
        """
        Build the final MCP response dictionary.
        
        Returns:
            MCP-compliant response dictionary
        """
        result: Dict[str, Any] = {}
        
        # Add content if any
        if self._content:
            result["content"] = self._content
        
        # Add structured content if provided
        if self._structured_content is not None:
            result["structuredContent"] = self._structured_content
        
        # Add error flag if this is an error
        if self._is_error:
            result["isError"] = True
        
        # Add progress if provided
        if self._progress is not None:
            result["progress"] = self._progress
        
        # Add metadata if provided
        if self._meta:
            result["_meta"] = self._meta
        
        return result
    
    def __call__(self) -> Dict[str, Any]:
        """
        Allow the ToolResult to be called like a function to build the response.
        
        Returns:
            MCP-compliant response dictionary
        """
        return self.build()
    
    # Static factory methods for convenience
    @staticmethod
    def simple_text(text: str) -> Dict[str, Any]:
        """
        Create a simple text response (convenience method).
        
        Args:
            text: The text content
            
        Returns:
            MCP-compliant response dictionary
        """
        return ToolResult().text(text).build()
    
    @staticmethod
    def simple_error(message: str, details: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Create a simple error response (convenience method).
        
        Args:
            message: Error message
            details: Optional error details
            
        Returns:
            MCP-compliant response dictionary
        """
        return ToolResult().error(message, details).build()


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
    result: List[str] = []
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
            
            # Log all requests for debugging
            print(f"[DEBUG] Method: {method}, Path: '{path}', URI: '{request.uri}'")
            
            # Handle GET / - return tool metadata
            if method == "GET" and (path == "/" or path == ""):
                metadata: List[Dict[str, Any]] = []
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
                clean_metadata: List[Dict[str, Any]] = []
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
                tool_entry: Optional[Dict[str, Any]] = None
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
            error_response = {
                "error": {
                    "code": -32601,
                    "message": "Method not allowed"
                }
            }
            return Response(
                405,
                {"content-type": "application/json", "allow": "GET, POST"},
                bytes(json.dumps(error_response), "utf-8")
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