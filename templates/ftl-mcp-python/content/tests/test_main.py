"""Tests for {{project-name}} MCP tool."""

import pytest
from src.main import example_tool


def test_example_tool():
    """Test the example tool."""
    # Test with a message
    result = example_tool("Hello, World!")
    assert result == "Processed: Hello, World!"
    
    # Test with empty message
    result = example_tool("")
    assert result == "Processed: "
    
    # Test with special characters
    result = example_tool("Test@123!")
    assert result == "Processed: Test@123!"


# Add more tests here as you implement your tools
# Example test for dict return:
# def test_another_tool():
#     from src.main import another_tool
#     result = another_tool("hello", 3)
#     assert result == {
#         "original": "hello",
#         "repeated": "hellohellohello",
#         "count": 3
#     }


# Example async test (if using async tools):
# @pytest.mark.asyncio
# async def test_async_tool():
#     from src.main import async_tool
#     result = await async_tool(["item1", "item2", "item3"])
#     assert result == {
#         "items": ["item1", "item2", "item3"],
#         "results": ["processed_item1", "processed_item2", "processed_item3"],
#         "count": 3
#     }