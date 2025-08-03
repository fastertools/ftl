"""Tests for ftl_sdk module."""

import json
from typing import Any, Dict

import pytest
from spin_sdk.http import Request, Response

from ftl_sdk import (
    FTL,
    ToolContent,
    ToolResponse,
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
        
        # Should automatically convert int result to text response
        expected = ToolResponse.text("8")
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
        
        # Should automatically convert to text response
        expected = ToolResponse.text("Echo: Hello")
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
        
        # Should automatically convert boolean to text
        expected = ToolResponse.text("True")
        assert result == expected