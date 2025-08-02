"""Tests for {{project-name}} MCP tool."""

import pytest
from src.main import example_tool_handler


def test_example_tool_handler():
    """Test the example tool handler."""
    # Test with a message
    result = example_tool_handler({"message": "Hello, World!"})
    assert result.content == "Processed: Hello, World!"
    
    # Test with empty input
    result = example_tool_handler({})
    assert result.content == "Processed: "
    
    # Test with missing message key
    result = example_tool_handler({"other": "value"})
    assert result.content == "Processed: "


# Add more tests here as you implement your tools
# Example async test (if needed):
# @pytest.mark.asyncio
# async def test_async_tool():
#     result = await async_tool_handler({"param": "value"})
#     assert result.content == "Expected output"