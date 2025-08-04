"""
FTL SDK for Python - Decorator-based API.

This module provides a modern, decorator-based API for creating MCP tools
that compile to WebAssembly.
"""

import asyncio
import inspect
import json
from collections.abc import Callable
from functools import wraps
from typing import Any, TypeVar, Union, get_type_hints

from spin_sdk import http
from spin_sdk.http import PollLoop, Request, Response

# Import response utilities
from .response import ToolResponse, _camel_to_snake

# Import validation utilities
from .validation import OutputSchemaGenerator

# Type definitions
T = TypeVar("T")
ToolFunction = TypeVar("ToolFunction", bound=Callable[..., Any])


class FTL:
    """
    Main FTL application class providing decorator-based tool registration.

    This class provides a central namespace
    for all MCP operations through decorators.

    Example:
        ftl = FTL()

        @ftl.tool
        def echo(message: str) -> str:
            '''Echo the input message'''
            return f"Echo: {message}"

        Handler = ftl.create_handler()
    """

    def __init__(self) -> None:
        """Initialize FTL instance with empty tool registry."""
        self._tools: dict[str, dict[str, Any]] = {}
        self._tool_functions: dict[str, Callable] = {}

    def tool(
        self,
        func: ToolFunction | None = None,
        *,
        name: str | None = None,
        description: str | None = None,
        annotations: dict[str, Any] | None = None,
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
            if name:
                # Convert provided camelCase names to snake_case
                tool_name = _camel_to_snake(name)
            else:
                tool_name = f.__name__
            tool_description = description or inspect.getdoc(f) or ""

            # Get type hints for schema generation
            hints = get_type_hints(f)
            signature = inspect.signature(f)

            # Generate input schema from parameters
            input_schema = self._generate_input_schema(signature, hints)

            # Generate output schema from return type annotation
            return_type = hints.get("return", None)
            output_schema = None
            if return_type is not None:
                output_schema = OutputSchemaGenerator.generate_from_type(return_type)

            # Store the tool definition
            tool_definition = {
                "name": tool_name,
                "description": tool_description,
                "inputSchema": input_schema,
                "annotations": annotations,
                "handler": self._create_handler_wrapper(f, hints, output_schema),
            }

            # Only add outputSchema if it was generated
            if output_schema is not None:
                tool_definition["outputSchema"] = output_schema

            self._tools[tool_name] = tool_definition

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
            if param_name == "self":
                continue

            # Get type hint or default to Any
            param_type = hints.get(param_name, Any)

            # Convert Python type to JSON Schema type
            json_type = self._python_type_to_json_schema(param_type)
            properties[param_name] = json_type

            # Check if parameter is required (no default value)
            if param.default == inspect.Parameter.empty:
                required.append(param_name)

        schema = {"type": "object", "properties": properties}

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
            type(None): {"type": "null"},
        }

        # Handle Optional types
        origin = getattr(python_type, "__origin__", None)
        if origin is Union:
            args = getattr(python_type, "__args__", ())
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
            args = getattr(python_type, "__args__", ())
            if args:
                return {"type": "array", "items": self._python_type_to_json_schema(args[0])}
            return {"type": "array"}

        # Handle Dict types
        if origin is dict:
            return {"type": "object"}

        # Default mapping
        return type_map.get(python_type, {"type": "object"})

    # Type validators for JSON Schema types
    _TYPE_VALIDATORS = {
        "string": lambda x: isinstance(x, str),
        "integer": lambda x: isinstance(x, int),
        "number": lambda x: isinstance(x, int | float),
        "boolean": lambda x: isinstance(x, bool),
        "object": lambda x: isinstance(x, dict),
        "array": lambda x: isinstance(x, list),
    }

    # Human-readable type names for error messages
    _TYPE_NAMES = {
        "string": "string",
        "integer": "integer",
        "number": "number",
        "boolean": "boolean",
        "object": "object/dict",
        "array": "array/list",
    }

    def _validate_type(self, output: Any, expected_type: str) -> None:
        """
        Validate that output matches expected JSON Schema type.

        Args:
            output: The value to validate
            expected_type: The expected JSON Schema type name

        Raises:
            ValueError: If output doesn't match expected type
        """
        validator = self._TYPE_VALIDATORS.get(expected_type)
        if validator and not validator(output):
            type_name = self._TYPE_NAMES.get(expected_type, expected_type)
            raise ValueError(f"Expected {type_name}, got {type(output).__name__}")

    def _validate_output_against_schema(self, output: Any, schema: dict[str, Any] | None) -> Any:
        """
        Validate output against its schema and apply wrapping if needed.

        This method ensures that outputs match their declared schemas and
        automatically wraps primitive values in {"result": value} format
        when indicated by the x-ftl-wrapped flag.

        Args:
            output: The raw output from the tool function
            schema: The output schema (may be None if no return type)

        Returns:
            The validated/wrapped output

        Raises:
            ValueError: If output doesn't match the declared schema
        """
        if not schema:
            return output

        schema_type = schema.get("type")

        # Handle wrapped primitives (x-ftl-wrapped indicates MCP wrapping needed)
        if schema.get("x-ftl-wrapped") and schema_type == "object":
            # This means we're expecting a primitive wrapped in {"result": value}
            # First validate the primitive type
            expected_primitive_schema = schema.get("properties", {}).get("result", {})
            expected_type = expected_primitive_schema.get("type")

            # Validate the primitive type
            if expected_type:
                self._validate_type(output, expected_type)

            if not isinstance(output, dict) or "result" not in output:
                # Auto-wrap the primitive value
                return {"result": output}

        # Basic type validation
        if schema_type:
            self._validate_type(output, schema_type)

        return output

    def _convert_result_to_toolresult(self, result: Any) -> dict[str, Any]:
        """
        Convert any function return value to MCP response format.

        This implements automatic conversion where functions
        can return basic Python types and the framework handles MCP formatting.

        Args:
            result: Any return value from a tool function

        Returns:
            Dict in MCP response format with content and optional structured_content
        """
        # If already in MCP format, pass through
        if isinstance(result, dict) and "content" in result:
            return result

        # Check if result was wrapped by validation (for primitives)
        if isinstance(result, dict) and "result" in result and len(result) == 1:
            # This is a wrapped primitive - extract the value for text content
            wrapped_value = result["result"]
            if isinstance(wrapped_value, str):
                return ToolResponse.with_structured(wrapped_value, result)
            else:
                return ToolResponse.with_structured(str(wrapped_value), result)

        # Handle different return types automatically
        if isinstance(result, str):
            # String -> text content
            return ToolResponse.text(result)
        elif isinstance(result, dict | list):
            # Structured data -> both text and structured content
            return ToolResponse.with_structured(json.dumps(result, indent=2), result)
        elif isinstance(result, int | float | bool):
            # Basic types -> convert to string content
            return ToolResponse.text(str(result))
        elif result is None:
            # None -> empty text content
            return ToolResponse.text("")
        else:
            # Everything else -> string representation
            return ToolResponse.text(str(result))

    def _create_handler_wrapper(
        self, func: Callable, hints: dict[str, type], output_schema: dict[str, Any] | None
    ) -> Callable[[dict[str, Any]], Any]:
        """Create a wrapper that converts MCP input to function parameters and validates output."""

        # Check if the function is async
        if inspect.iscoroutinefunction(func):

            @wraps(func)
            async def async_wrapper(input_data: dict[str, Any]) -> dict[str, Any]:
                try:
                    # Call the async function with await
                    result = await func(**input_data)

                    # Validate and potentially wrap the output according to schema
                    validated_result = self._validate_output_against_schema(result, output_schema)

                    # Automatically convert any return type to MCP format
                    return self._convert_result_to_toolresult(validated_result)

                except Exception as e:
                    return ToolResponse.error(f"Tool execution failed: {str(e)}")

            return async_wrapper
        else:
            # Original sync wrapper code
            @wraps(func)
            def wrapper(input_data: dict[str, Any]) -> dict[str, Any]:
                try:
                    # Call the original function with input data as kwargs
                    result = func(**input_data)

                    # Validate and potentially wrap the output according to schema
                    validated_result = self._validate_output_against_schema(result, output_schema)

                    # Automatically convert any return type to MCP format
                    return self._convert_result_to_toolresult(validated_result)

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
                        tool_metadata = {
                            "name": tool_name,
                            "description": tool.get("description", ""),
                            "inputSchema": tool.get("inputSchema", {"type": "object"}),
                            "annotations": tool.get("annotations"),
                        }

                        # Add outputSchema if present
                        if "outputSchema" in tool:
                            tool_metadata["outputSchema"] = tool["outputSchema"]

                        metadata.append(tool_metadata)

                    # Remove None values
                    clean_metadata = [{k: v for k, v in item.items() if v is not None} for item in metadata]

                    return Response(
                        200, {"content-type": "application/json"}, bytes(json.dumps(clean_metadata), "utf-8")
                    )

                # Handle POST /{tool_name} - execute tool
                elif method == "POST":
                    tool_name = path.lstrip("/")

                    if tool_name not in tools:
                        error_response = ToolResponse.error(f"Tool '{tool_name}' not found")
                        return Response(
                            404, {"content-type": "application/json"}, bytes(json.dumps(error_response), "utf-8")
                        )

                    # Execute the tool
                    try:
                        body = request.body.decode("utf-8") if request.body else "{}"
                        input_data = json.loads(body)
                        handler = tools[tool_name]["handler"]

                        # Execute handler
                        result = handler(input_data)

                        # If it's a coroutine, we need to run it
                        if inspect.iscoroutine(result):
                            # Use WASM-compatible PollLoop for async execution
                            loop = PollLoop()
                            asyncio.set_event_loop(loop)
                            result = loop.run_until_complete(result)

                        return Response(200, {"content-type": "application/json"}, bytes(json.dumps(result), "utf-8"))
                    except Exception as e:
                        error_response = ToolResponse.error(f"Tool execution failed: {str(e)}")
                        return Response(
                            400, {"content-type": "application/json"}, bytes(json.dumps(error_response), "utf-8")
                        )

                # Method not allowed
                error_response = {"error": {"code": -32601, "message": "Method not allowed"}}
                return Response(
                    405,
                    {"content-type": "application/json", "allow": "GET, POST"},
                    bytes(json.dumps(error_response), "utf-8"),
                )

        return IncomingHandler
