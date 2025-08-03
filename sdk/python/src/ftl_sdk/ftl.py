"""
FTL SDK for Python - FastMCP-style decorator-based API.

This module provides a modern, decorator-based API for creating MCP tools
that compile to WebAssembly, following the FastMCP patterns.
"""

import inspect
import json
from collections.abc import Callable
from functools import wraps
from typing import Any, TypeVar, Union, get_type_hints

from spin_sdk import http
from spin_sdk.http import Request, Response

# Import response utilities
from .response import ToolResponse

# Type definitions
T = TypeVar('T')
ToolFunction = TypeVar('ToolFunction', bound=Callable[..., Any])


class FTL:
    """
    Main FTL application class providing decorator-based tool registration.
    
    This class follows the FastMCP pattern of providing a central namespace
    for all MCP operations through decorators.
    
    Example:
        ftl = FTL()
        
        @ftl.tool
        def echo(message: str) -> str:
            '''Echo the input message'''
            return f"Echo: {message}"
        
        Handler = ftl.create_handler()
    """
    
    def __init__(self):
        """Initialize FTL instance with empty tool registry."""
        self._tools: dict[str, dict[str, Any]] = {}
        self._tool_functions: dict[str, Callable] = {}
    
    def tool(
        self,
        func: ToolFunction | None = None,
        *,
        name: str | None = None,
        description: str | None = None,
        annotations: dict[str, Any] | None = None
    ) -> ToolFunction | Callable[[ToolFunction], ToolFunction]:
        """
        Decorator for registering a function as an MCP tool.
        
        This decorator automatically extracts:
        - Function name (or uses provided name)
        - Docstring as description (or uses provided description)
        - Type hints for JSON schema generation
        - Parameter information for input schema
        
        Args:
            func: The function to register (when used without parentheses)
            name: Optional override for tool name (defaults to function name)
            description: Optional override for description (defaults to docstring)
            annotations: Optional MCP annotations for tool behavior
        
        Returns:
            The decorated function unchanged (for stacking decorators)
        
        Example:
            @ftl.tool
            def add(a: int, b: int) -> int:
                '''Add two numbers'''
                return a + b
            
            @ftl.tool(name="custom_name", annotations={"priority": "high"})
            def process(data: str) -> str:
                return data.upper()
        """
        def decorator(f: ToolFunction) -> ToolFunction:
            # Extract tool metadata
            tool_name = name or f.__name__
            tool_description = description or inspect.getdoc(f) or ""
            
            # Get type hints for schema generation
            hints = get_type_hints(f)
            signature = inspect.signature(f)
            
            # Generate input schema from parameters
            input_schema = self._generate_input_schema(signature, hints)
            
            # Store the tool definition
            self._tools[tool_name] = {
                "name": tool_name,
                "description": tool_description,
                "inputSchema": input_schema,
                "annotations": annotations,
                "handler": self._create_handler_wrapper(f, hints)
            }
            
            # Store the original function for direct access
            self._tool_functions[tool_name] = f
            
            # Return the function unchanged
            return f
        
        # Support both @ftl.tool and @ftl.tool()
        if func is None:
            return decorator
        else:
            return decorator(func)
    
    def _generate_input_schema(self, signature: inspect.Signature, hints: dict[str, type]) -> dict[str, Any]:
        """Generate JSON Schema from function signature and type hints."""
        properties: dict[str, Any] = {}
        required: list[str] = []
        
        for param_name, param in signature.parameters.items():
            if param_name == 'self':
                continue
                
            # Get type hint or default to Any
            param_type = hints.get(param_name, Any)
            
            # Convert Python type to JSON Schema type
            json_type = self._python_type_to_json_schema(param_type)
            properties[param_name] = json_type
            
            # Check if parameter is required (no default value)
            if param.default == inspect.Parameter.empty:
                required.append(param_name)
        
        schema = {
            "type": "object",
            "properties": properties
        }
        
        if required:
            schema["required"] = required
            
        return schema
    
    def _python_type_to_json_schema(self, python_type: type) -> dict[str, Any]:
        """Convert Python type hint to JSON Schema type."""
        # Basic type mapping
        type_map = {
            str: {"type": "string"},
            int: {"type": "integer"},
            float: {"type": "number"},
            bool: {"type": "boolean"},
            list: {"type": "array"},
            dict: {"type": "object"},
            type(None): {"type": "null"}
        }
        
        # Handle Optional types
        origin = getattr(python_type, '__origin__', None)
        if origin is Union:
            args = python_type.__args__
            # Check if it's Optional (Union with None)
            if type(None) in args:
                # Get the non-None type
                non_none_types = [t for t in args if t is not type(None)]
                if len(non_none_types) == 1:
                    base_schema = self._python_type_to_json_schema(non_none_types[0])
                    # Make it nullable
                    if "type" in base_schema:
                        base_schema["type"] = [base_schema["type"], "null"]
                    return base_schema
        
        # Handle List types
        if origin is list:
            args = getattr(python_type, '__args__', ())
            if args:
                return {
                    "type": "array",
                    "items": self._python_type_to_json_schema(args[0])
                }
            return {"type": "array"}
        
        # Handle Dict types
        if origin is dict:
            return {"type": "object"}
        
        # Default mapping
        return type_map.get(python_type, {"type": "object"})
    
    def _create_handler_wrapper(self, func: Callable, hints: dict[str, type]) -> Callable[[dict[str, Any]], dict[str, Any]]:
        """Create a wrapper that converts MCP input to function parameters."""
        @wraps(func)
        def wrapper(input_data: dict[str, Any]) -> dict[str, Any]:
            try:
                # Call the original function with input data as kwargs
                result = func(**input_data)
                
                # Convert result to MCP response format
                if isinstance(result, dict) and "content" in result:
                    # Already in MCP format
                    return result
                elif isinstance(result, str):
                    # Simple string response
                    return ToolResponse.text(result)
                elif isinstance(result, dict | list):
                    # Structured data response
                    return ToolResponse.with_structured(
                        json.dumps(result, indent=2),
                        result
                    )
                else:
                    # Convert to string
                    return ToolResponse.text(str(result))
                    
            except Exception as e:
                return ToolResponse.error(f"Tool execution failed: {str(e)}")
        
        return wrapper
    
    def create_handler(self) -> type:
        """
        Create a Spin HTTP handler from registered tools.
        
        This method generates a handler class that implements the MCP protocol
        for all registered tools.
        
        Returns:
            A Spin IncomingHandler class
        
        Example:
            ftl = FTL()
            
            @ftl.tool
            def my_tool(input: str) -> str:
                return input.upper()
            
            Handler = ftl.create_handler()
        """
        tools = self._tools
        
        class IncomingHandler(http.IncomingHandler):
            def handle_request(self, request: Request) -> Response:
                path = request.uri
                method = request.method
                
                # Handle GET / - return tool metadata
                if method == "GET" and (path == "/" or path == ""):
                    metadata: list[dict[str, Any]] = []
                    for tool_name, tool in tools.items():
                        metadata.append({
                            "name": tool_name,
                            "description": tool.get("description", ""),
                            "inputSchema": tool.get("inputSchema", {"type": "object"}),
                            "annotations": tool.get("annotations")
                        })
                    
                    # Remove None values
                    clean_metadata = [
                        {k: v for k, v in item.items() if v is not None}
                        for item in metadata
                    ]
                    
                    return Response(
                        200,
                        {"content-type": "application/json"},
                        bytes(json.dumps(clean_metadata), "utf-8")
                    )
                
                # Handle POST /{tool_name} - execute tool
                elif method == "POST":
                    tool_name = path.lstrip('/')
                    
                    if tool_name not in tools:
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
                        handler = tools[tool_name]["handler"]
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


# Convenience function for backward compatibility
def tool(
    func: ToolFunction | None = None,
    *,
    name: str | None = None,
    description: str | None = None,
    annotations: dict[str, Any] | None = None
) -> ToolFunction | Callable[[ToolFunction], ToolFunction]:
    """
    Standalone decorator for tools when using a global FTL instance.
    
    This is a convenience function that uses a default global FTL instance.
    For more control, create your own FTL instance.
    
    Example:
        @tool
        def echo(message: str) -> str:
            '''Echo the input'''
            return message
    """
    # Use the default global instance
    return _default_ftl.tool(func, name=name, description=description, annotations=annotations)


# Create a default global FTL instance
_default_ftl = FTL()