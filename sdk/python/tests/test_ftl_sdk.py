"""Tests for ftl_sdk module."""

import json
from typing import Any, Dict

import pytest
from spin_sdk.http import Request, Response

from ftl_sdk import (
    FTL,
    ToolContent,
    ToolResponse,
    ToolResult,
    create_tools,
    is_audio_content,
    is_image_content,
    is_resource_content,
    is_text_content,
)


class MockRequest:
    """Mock HTTP request for testing."""

    def __init__(self, method: str, uri: str, body: bytes = b"{}"):
        self.method = method
        self.uri = uri
        self.body = body


def test_tool_response_text() -> None:
    """Test text response creation."""
    response = ToolResponse.text("Hello, world!")
    assert response == {
        "content": [{"type": "text", "text": "Hello, world!"}]
    }


def test_tool_response_error() -> None:
    """Test error response creation."""
    response = ToolResponse.error("Something went wrong")
    assert response == {
        "content": [{"type": "text", "text": "Something went wrong"}],
        "isError": True,
    }


def test_tool_response_with_structured() -> None:
    """Test response with structured content."""
    response = ToolResponse.with_structured("Result", {"value": 42})
    assert response == {
        "content": [{"type": "text", "text": "Result"}],
        "structuredContent": {"value": 42},
    }


def test_tool_content_text() -> None:
    """Test text content creation."""
    content = ToolContent.text("Hello")
    assert content == {"type": "text", "text": "Hello"}

    # With annotations
    content = ToolContent.text("Hello", {"priority": 0.8})
    assert content == {
        "type": "text",
        "text": "Hello",
        "annotations": {"priority": 0.8},
    }


def test_tool_content_image() -> None:
    """Test image content creation."""
    content = ToolContent.image("base64data", "image/png")
    assert content == {
        "type": "image",
        "data": "base64data",
        "mimeType": "image/png",
    }


def test_tool_content_audio() -> None:
    """Test audio content creation."""
    content = ToolContent.audio("base64data", "audio/wav")
    assert content == {
        "type": "audio",
        "data": "base64data",
        "mimeType": "audio/wav",
    }


def test_tool_content_resource() -> None:
    """Test resource content creation."""
    resource = {"uri": "file:///example.txt"}
    content = ToolContent.resource(resource)
    assert content == {"type": "resource", "resource": resource}


def test_content_type_guards() -> None:
    """Test content type guard functions."""
    text_content = {"type": "text", "text": "Hello"}
    image_content = {"type": "image", "data": "...", "mimeType": "image/png"}
    audio_content = {"type": "audio", "data": "...", "mimeType": "audio/wav"}
    resource_content = {"type": "resource", "resource": {"uri": "..."}}

    assert is_text_content(text_content) is True
    assert is_text_content(image_content) is False

    assert is_image_content(image_content) is True
    assert is_image_content(text_content) is False

    assert is_audio_content(audio_content) is True
    assert is_audio_content(text_content) is False

    assert is_resource_content(resource_content) is True
    assert is_resource_content(text_content) is False


def test_create_tools_metadata() -> None:
    """Test that create_tools returns correct metadata."""
    Handler = create_tools({
        "echo": {
            "description": "Echo the input",
            "inputSchema": {
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"],
            },
            "handler": lambda input: ToolResponse.text(f"Echo: {input['message']}"),
        }
    })

    handler = Handler()
    request = MockRequest("GET", "/")
    response = handler.handle_request(request)

    assert response.status == 200
    assert response.headers["content-type"] == "application/json"

    metadata = json.loads(response.body.decode("utf-8"))
    assert len(metadata) == 1
    assert metadata[0]["name"] == "echo"
    assert metadata[0]["description"] == "Echo the input"
    assert "inputSchema" in metadata[0]


def test_create_tools_execution() -> None:
    """Test tool execution."""
    Handler = create_tools({
        "echo": {
            "description": "Echo the input",
            "inputSchema": {
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"],
            },
            "handler": lambda input: ToolResponse.text(f"Echo: {input['message']}"),
        }
    })

    handler = Handler()
    request = MockRequest(
        "POST", "/echo", json.dumps({"message": "Hello"}).encode("utf-8")
    )
    response = handler.handle_request(request)

    assert response.status == 200
    result = json.loads(response.body.decode("utf-8"))
    assert result["content"][0]["text"] == "Echo: Hello"


def test_create_tools_camel_to_snake() -> None:
    """Test camelCase to snake_case conversion."""
    Handler = create_tools({
        "reverseText": {  # camelCase
            "description": "Reverse text",
            "handler": lambda input: ToolResponse.text("reversed"),
        }
    })

    handler = Handler()
    request = MockRequest("GET", "/")
    response = handler.handle_request(request)

    metadata = json.loads(response.body.decode("utf-8"))
    assert metadata[0]["name"] == "reverse_text"  # Converted to snake_case


def test_create_tools_name_override() -> None:
    """Test explicit name override."""
    Handler = create_tools({
        "reverseText": {
            "name": "reverse",  # Override name
            "description": "Reverse text",
            "handler": lambda input: ToolResponse.text("reversed"),
        }
    })

    handler = Handler()
    request = MockRequest("GET", "/")
    response = handler.handle_request(request)

    metadata = json.loads(response.body.decode("utf-8"))
    assert metadata[0]["name"] == "reverse"  # Uses override


def test_create_tools_not_found() -> None:
    """Test 404 for unknown tool."""
    Handler = create_tools({
        "echo": {
            "description": "Echo",
            "handler": lambda input: ToolResponse.text("echo"),
        }
    })

    handler = Handler()
    request = MockRequest("POST", "/unknown")
    response = handler.handle_request(request)

    assert response.status == 404
    result = json.loads(response.body.decode("utf-8"))
    assert result["content"][0]["text"] == "Tool 'unknown' not found"


def test_create_tools_error_handling() -> None:
    """Test error handling in tool execution."""

    def failing_handler(input: Dict[str, Any]) -> Dict[str, Any]:
        raise ValueError("Test error")

    Handler = create_tools({
        "fail": {
            "description": "Failing tool",
            "handler": failing_handler,
        }
    })

    handler = Handler()
    request = MockRequest("POST", "/fail")
    response = handler.handle_request(request)

    assert response.status == 400
    result = json.loads(response.body.decode("utf-8"))
    assert "Tool execution failed" in result["content"][0]["text"]


def test_create_tools_method_not_allowed() -> None:
    """Test 405 for unsupported methods."""
    Handler = create_tools({})

    handler = Handler()
    request = MockRequest("DELETE", "/")
    response = handler.handle_request(request)

    assert response.status == 405
    assert response.headers["allow"] == "GET, POST"


# Tests for new FTL class and automatic conversion
class TestFTLAutomaticConversion:
    """Test the new FTL class with automatic return value conversion."""
    
    def test_convert_result_string(self):
        """Test automatic conversion of string return values."""
        ftl = FTL()
        result = ftl._convert_result_to_toolresult("Hello, world!")
        expected = ToolResponse.text("Hello, world!")
        assert result == expected
    
    def test_convert_result_integer(self):
        """Test automatic conversion of integer return values."""
        ftl = FTL()
        result = ftl._convert_result_to_toolresult(42)
        expected = ToolResponse.text("42")
        assert result == expected
    
    def test_convert_result_float(self):
        """Test automatic conversion of float return values."""
        ftl = FTL()
        result = ftl._convert_result_to_toolresult(3.14159)
        expected = ToolResponse.text("3.14159")
        assert result == expected
    
    def test_convert_result_boolean(self):
        """Test automatic conversion of boolean return values."""
        ftl = FTL()
        result = ftl._convert_result_to_toolresult(True)
        expected = ToolResponse.text("True")
        assert result == expected
        
        result = ftl._convert_result_to_toolresult(False)
        expected = ToolResponse.text("False")
        assert result == expected
    
    def test_convert_result_dict(self):
        """Test automatic conversion of dict return values."""
        ftl = FTL()
        data = {"name": "test", "value": 42}
        result = ftl._convert_result_to_toolresult(data)
        expected = ToolResponse.with_structured(
            json.dumps(data, indent=2),
            data
        )
        assert result == expected
    
    def test_convert_result_list(self):
        """Test automatic conversion of list return values."""
        ftl = FTL()
        data = ["item1", "item2", "item3"]
        result = ftl._convert_result_to_toolresult(data)
        expected = ToolResponse.with_structured(
            json.dumps(data, indent=2),
            data
        )
        assert result == expected
    
    def test_convert_result_none(self):
        """Test automatic conversion of None return values."""
        ftl = FTL()
        result = ftl._convert_result_to_toolresult(None)
        expected = ToolResponse.text("")
        assert result == expected
    
    def test_convert_result_already_mcp_format(self):
        """Test pass-through of already-formatted MCP responses."""
        ftl = FTL()
        mcp_response = {"content": [{"type": "text", "text": "Already formatted"}]}
        result = ftl._convert_result_to_toolresult(mcp_response)
        assert result == mcp_response
    
    def test_ftl_tool_decorator_with_automatic_conversion(self):
        """Test @ftl.tool decorator with automatic conversion in action."""
        ftl = FTL()
        
        @ftl.tool
        def add_numbers(a: int, b: int) -> int:
            """Add two numbers."""
            return a + b
        
        # Check that the tool was registered
        assert "add_numbers" in ftl._tools
        
        # Test the handler directly
        handler = ftl._tools["add_numbers"]["handler"]
        result = handler({"a": 5, "b": 3})
        
        # Should automatically convert int result to text response with structured content
        # When return type is annotated, primitives are wrapped
        expected = ToolResponse.with_structured("8", {"result": 8})
        assert result == expected
    
    def test_ftl_tool_decorator_with_dict_return(self):
        """Test @ftl.tool decorator with dict return value."""
        ftl = FTL()
        
        @ftl.tool
        def get_user_info(name: str, age: int) -> dict:
            """Get user information."""
            return {"name": name, "age": age, "status": "active"}
        
        # Test the handler
        handler = ftl._tools["get_user_info"]["handler"]
        result = handler({"name": "Alice", "age": 30})
        
        # Should automatically convert to structured response
        data = {"name": "Alice", "age": 30, "status": "active"}
        expected = ToolResponse.with_structured(
            json.dumps(data, indent=2),
            data
        )
        assert result == expected
    
    def test_ftl_tool_decorator_with_string_return(self):
        """Test @ftl.tool decorator with string return value."""
        ftl = FTL()
        
        @ftl.tool
        def echo_message(message: str) -> str:
            """Echo a message."""
            return f"Echo: {message}"
        
        # Test the handler
        handler = ftl._tools["echo_message"]["handler"]
        result = handler({"message": "Hello"})
        
        # Should automatically convert to text response with structured content
        # When return type is annotated, strings are wrapped
        expected = ToolResponse.with_structured("Echo: Hello", {"result": "Echo: Hello"})
        assert result == expected
        
    def test_ftl_tool_decorator_with_boolean_return(self):
        """Test @ftl.tool decorator with boolean return value."""
        ftl = FTL()
        
        @ftl.tool
        def is_even(number: int) -> bool:
            """Check if number is even."""
            return number % 2 == 0
        
        # Test the handler
        handler = ftl._tools["is_even"]["handler"]
        result = handler({"number": 4})
        
        # Should automatically convert boolean to text with structured content
        # When return type is annotated, booleans are wrapped
        expected = ToolResponse.with_structured("True", {"result": True})
        assert result == expected


# Tests for new ToolResult constructor pattern
class TestToolResultConstructor:
    """Test the new ToolResult simple constructor API."""
    
    def test_toolresult_string_content(self):
        """Test ToolResult with string content."""
        result = ToolResult("Hello world")
        assert result.content == [{"type": "text", "text": "Hello world"}]
        assert result.structured_content is None
    
    def test_toolresult_structured_content_only(self):
        """Test ToolResult with structured content only."""
        data = {"status": "success", "count": 42}
        result = ToolResult(structured_content=data)
        # When only structured_content provided, it's converted to content
        # The dict is passed to _convert_to_content which wraps it as a single content block
        assert result.content == [data]
        assert result.structured_content == data
    
    def test_toolresult_both_content_and_structured(self):
        """Test ToolResult with both content and structured content."""
        data = {"status": "success"}
        result = ToolResult("Process completed", data)
        assert result.content == [{"type": "text", "text": "Process completed"}]
        assert result.structured_content == data
    
    def test_toolresult_list_content(self):
        """Test ToolResult with list of content blocks."""
        content_blocks = [
            {"type": "text", "text": "Result"},
            {"type": "image", "data": "base64data", "mimeType": "image/png"}
        ]
        result = ToolResult(content_blocks)
        assert result.content == content_blocks
        assert result.structured_content is None
    
    def test_toolresult_dict_content(self):
        """Test ToolResult with single content block dict."""
        content_block = {"type": "text", "text": "Single block"}
        result = ToolResult(content_block)
        assert result.content == [content_block]
        assert result.structured_content is None
    
    def test_toolresult_none_parameters_raises_error(self):
        """Test that ToolResult raises error when both parameters are None."""
        with pytest.raises(ValueError, match="Either content or structured_content must be provided"):
            ToolResult(None, None)
    
    def test_toolresult_to_mcp_result_content_only(self):
        """Test to_mcp_result() with content only."""
        result = ToolResult("Hello")
        mcp_result = result.to_mcp_result()
        expected = [{"type": "text", "text": "Hello"}]
        assert mcp_result == expected
    
    def test_toolresult_to_mcp_result_with_structured(self):
        """Test to_mcp_result() with both content and structured content."""
        data = {"count": 5}
        result = ToolResult("Completed", data)
        mcp_result = result.to_mcp_result()
        expected_content = [{"type": "text", "text": "Completed"}]
        assert mcp_result == (expected_content, data)
    
    def test_toolresult_integer_content_conversion(self):
        """Test ToolResult with integer content gets converted to string."""
        result = ToolResult(42)
        assert result.content == [{"type": "text", "text": "42"}]
    
    def test_toolresult_complex_object_content_conversion(self):
        """Test ToolResult with complex object gets converted to string."""
        class CustomObject:
            def __str__(self):
                return "custom_object_string"
        
        obj = CustomObject()
        result = ToolResult(obj)
        assert result.content == [{"type": "text", "text": "custom_object_string"}]


# Tests for output schema validation
class TestOutputSchemaValidation:
    """Test output schema generation and validation."""
    
    def test_output_schema_generation_primitive_types(self):
        """Test that primitive return types generate wrapped schemas."""
        ftl = FTL()
        
        @ftl.tool
        def get_count() -> int:
            """Get a count."""
            return 42
        
        # Check generated output schema
        tool_def = ftl._tools["get_count"]
        assert "outputSchema" in tool_def
        assert tool_def["outputSchema"] == {
            "type": "object",
            "properties": {
                "result": {"type": "integer"}
            },
            "required": ["result"],
            "x-ftl-wrapped": True
        }
    
    def test_output_schema_generation_string_type(self):
        """Test string return type generates wrapped schema."""
        ftl = FTL()
        
        @ftl.tool
        def get_message() -> str:
            """Get a message."""
            return "hello"
        
        tool_def = ftl._tools["get_message"]
        assert tool_def["outputSchema"] == {
            "type": "object",
            "properties": {
                "result": {"type": "string"}
            },
            "required": ["result"],
            "x-ftl-wrapped": True
        }
    
    def test_output_schema_generation_dict_type(self):
        """Test dict return type generates object schema without wrapping."""
        ftl = FTL()
        
        @ftl.tool
        def get_data() -> dict:
            """Get data."""
            return {"key": "value"}
        
        tool_def = ftl._tools["get_data"]
        assert tool_def["outputSchema"] == {"type": "object"}
    
    def test_output_schema_generation_no_return_type(self):
        """Test functions without return type have no output schema."""
        ftl = FTL()
        
        @ftl.tool
        def do_something(value: str):
            """Do something."""
            pass
        
        tool_def = ftl._tools["do_something"]
        assert "outputSchema" not in tool_def
    
    def test_primitive_wrapping_validation(self):
        """Test that primitive values are auto-wrapped for MCP compliance."""
        ftl = FTL()
        
        @ftl.tool
        def add_numbers(a: int, b: int) -> int:
            """Add two numbers."""
            return a + b
        
        handler = ftl._tools["add_numbers"]["handler"]
        result = handler({"a": 5, "b": 3})
        
        # Check structured content has wrapped value
        assert "structuredContent" in result
        assert result["structuredContent"] == {"result": 8}
        
        # Check text content shows the value
        assert result["content"][0]["text"] == "8"
    
    def test_string_wrapping_validation(self):
        """Test that string values are auto-wrapped."""
        ftl = FTL()
        
        @ftl.tool
        def greet(name: str) -> str:
            """Greet someone."""
            return f"Hello, {name}!"
        
        handler = ftl._tools["greet"]["handler"]
        result = handler({"name": "Alice"})
        
        # Check structured content has wrapped value
        assert "structuredContent" in result
        assert result["structuredContent"] == {"result": "Hello, Alice!"}
    
    def test_dict_passthrough_validation(self):
        """Test that dict values pass through without wrapping."""
        ftl = FTL()
        
        @ftl.tool
        def get_user_data() -> dict:
            """Get user data."""
            return {"id": 123, "name": "test"}
        
        handler = ftl._tools["get_user_data"]["handler"]
        result = handler({})
        
        # Dict should pass through directly as structured content
        assert result["structuredContent"] == {"id": 123, "name": "test"}
    
    def test_validation_type_error(self):
        """Test that type mismatches raise validation errors."""
        ftl = FTL()
        
        @ftl.tool
        def get_number() -> int:
            """Get a number."""
            return "not a number"  # Wrong type!
        
        handler = ftl._tools["get_number"]["handler"]
        result = handler({})
        
        # Should return error response
        assert result["content"][0]["text"].startswith("Tool execution failed:")
        assert "Expected integer" in result["content"][0]["text"]
    
    def test_boolean_wrapping_validation(self):
        """Test boolean return type validation and wrapping."""
        ftl = FTL()
        
        @ftl.tool
        def is_ready() -> bool:
            """Check if ready."""
            return True
        
        handler = ftl._tools["is_ready"]["handler"]
        result = handler({})
        
        # Should wrap boolean in result
        assert result["structuredContent"] == {"result": True}
        assert result["content"][0]["text"] == "True"
    
    def test_output_schema_in_metadata(self):
        """Test that output schemas appear in GET / metadata."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        def calculate(x: int) -> int:
            """Calculate something."""
            return x * 2
        
        @ftl.tool
        def get_config() -> dict:
            """Get configuration."""
            return {"debug": True}
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        request = MockRequest("GET", "/")
        
        # Run async handler
        loop = asyncio.new_event_loop()
        response = loop.run_until_complete(handler.handle_request(request))
        loop.close()
        
        metadata = json.loads(response.body.decode("utf-8"))
        
        # Find calculate tool
        calc_tool = next(t for t in metadata if t["name"] == "calculate")
        assert "outputSchema" in calc_tool
        assert calc_tool["outputSchema"]["x-ftl-wrapped"] is True
        
        # Find get_config tool
        config_tool = next(t for t in metadata if t["name"] == "get_config")
        assert "outputSchema" in config_tool
        assert config_tool["outputSchema"]["type"] == "object"
        assert "x-ftl-wrapped" not in config_tool["outputSchema"]


class TestAsyncSupport:
    """Tests for async function support in FTL SDK."""
    
    def test_async_tool_decorator(self):
        """Test that async functions can be decorated with @ftl.tool."""
        ftl = FTL()
        
        @ftl.tool
        async def fetch_data(url: str) -> dict:
            """Fetch data asynchronously."""
            # Simulate async operation
            return {"url": url, "status": "fetched"}
        
        # Verify tool was registered
        assert "fetch_data" in ftl._tools
        tool_def = ftl._tools["fetch_data"]
        assert tool_def["description"] == "Fetch data asynchronously."
        assert tool_def["inputSchema"]["properties"]["url"]["type"] == "string"
    
    def test_async_tool_execution(self):
        """Test async tool execution via handler."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        async def async_echo(message: str) -> str:
            """Echo message asynchronously."""
            # Simulate async delay
            await asyncio.sleep(0.001)
            return f"Async echo: {message}"
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        # Test async tool execution
        request = MockRequest("POST", "/async_echo", b'{"message": "Hello async"}')
        
        # Run async handler
        loop = asyncio.new_event_loop()
        response = loop.run_until_complete(handler.handle_request(request))
        loop.close()
        
        assert response.status == 200
        result = json.loads(response.body.decode("utf-8"))
        assert result["content"][0]["text"] == "Async echo: Hello async"
    
    def test_mixed_sync_async_tools(self):
        """Test FTL with both sync and async tools."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        def sync_add(a: int, b: int) -> int:
            """Add two numbers synchronously."""
            return a + b
        
        @ftl.tool
        async def async_multiply(x: int, y: int) -> int:
            """Multiply two numbers asynchronously."""
            await asyncio.sleep(0.001)
            return x * y
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        # Test sync tool
        sync_request = MockRequest("POST", "/sync_add", b'{"a": 3, "b": 4}')
        loop = asyncio.new_event_loop()
        sync_response = loop.run_until_complete(handler.handle_request(sync_request))
        
        assert sync_response.status == 200
        sync_result = json.loads(sync_response.body.decode("utf-8"))
        assert sync_result["content"][0]["text"] == "7"
        
        # Test async tool
        async_request = MockRequest("POST", "/async_multiply", b'{"x": 5, "y": 6}')
        async_response = loop.run_until_complete(handler.handle_request(async_request))
        loop.close()
        
        assert async_response.status == 200
        async_result = json.loads(async_response.body.decode("utf-8"))
        assert async_result["content"][0]["text"] == "30"
    
    def test_async_tool_with_dict_return(self):
        """Test async tool returning dict with automatic conversion."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        async def async_process(items: list[str]) -> dict:
            """Process items asynchronously."""
            await asyncio.sleep(0.001)
            processed = [item.upper() for item in items]
            return {"processed": processed, "count": len(processed)}
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        request = MockRequest("POST", "/async_process", b'{"items": ["hello", "world"]}')
        
        loop = asyncio.new_event_loop()
        response = loop.run_until_complete(handler.handle_request(request))
        loop.close()
        
        assert response.status == 200
        result = json.loads(response.body.decode("utf-8"))
        
        # Should have text content and structured content
        assert len(result["content"]) == 1
        assert result["content"][0]["type"] == "text"
        
        # Verify structured content
        assert "structuredContent" in result
        assert result["structuredContent"]["processed"] == ["HELLO", "WORLD"]
        assert result["structuredContent"]["count"] == 2
    
    def test_async_tool_error_handling(self):
        """Test error handling in async tools."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        async def async_fail(should_fail: bool) -> str:
            """Tool that might fail."""
            if should_fail:
                raise ValueError("Async failure!")
            return "Success"
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        # Test failure case
        request = MockRequest("POST", "/async_fail", b'{"should_fail": true}')
        
        loop = asyncio.new_event_loop()
        response = loop.run_until_complete(handler.handle_request(request))
        loop.close()
        
        assert response.status == 200
        result = json.loads(response.body.decode("utf-8"))
        assert result["isError"] is True
        assert "Async failure!" in result["content"][0]["text"]
    
    def test_async_tool_with_output_schema(self):
        """Test async tool with output schema validation."""
        import asyncio
        
        ftl = FTL()
        
        @ftl.tool
        async def async_calculate(x: int, y: int) -> int:
            """Calculate result asynchronously."""
            await asyncio.sleep(0.001)
            return x + y
        
        Handler = ftl.create_handler()
        handler = Handler()
        
        # Check metadata shows output schema
        meta_request = MockRequest("GET", "/")
        loop = asyncio.new_event_loop()
        meta_response = loop.run_until_complete(handler.handle_request(meta_request))
        
        metadata = json.loads(meta_response.body.decode("utf-8"))
        calc_tool = next(t for t in metadata if t["name"] == "async_calculate")
        
        assert "outputSchema" in calc_tool
        assert calc_tool["outputSchema"]["type"] == "object"
        assert calc_tool["outputSchema"]["x-ftl-wrapped"] is True
        
        # Test execution
        request = MockRequest("POST", "/async_calculate", b'{"x": 10, "y": 20}')
        response = loop.run_until_complete(handler.handle_request(request))
        loop.close()
        
        result = json.loads(response.body.decode("utf-8"))
        assert result["content"][0]["text"] == "30"
        # For primitive returns with schema, structured content is wrapped
        assert "structuredContent" in result
        assert result["structuredContent"]["result"] == 30