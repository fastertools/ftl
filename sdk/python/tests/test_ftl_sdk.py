"""Tests for ftl_sdk module."""

import json
from typing import Any, Dict

import pytest
from spin_sdk.http import Request, Response

from ftl_sdk import (
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