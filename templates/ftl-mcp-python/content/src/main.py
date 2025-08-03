"""
{{project-name}} - An FTL MCP tool written in Python.

This tool demonstrates how to create MCP tools using the FTL Python SDK.
"""

from ftl_sdk import FTL

# Create FTL application instance
ftl = FTL()


@ftl.tool
def example_tool(message: str) -> str:
    """
    Example tool that processes messages.
    
    Replace with your actual tool implementation.
    """
    # TODO: Implement your tool logic here
    return f"Processed: {message}"


# Example: Add more tools with type hints
# @ftl.tool
# def another_tool(text: str, count: int = 1) -> dict:
#     """Another example tool that returns structured data."""
#     return {
#         "original": text,
#         "repeated": text * count,
#         "count": count
#     }


# Example: Async tool (if needed)
# @ftl.tool
# async def async_tool(items: list[str]) -> dict:
#     """Example async tool that processes items concurrently."""
#     import asyncio
#     
#     async def process_item(item: str) -> str:
#         # Process each item (CPU-bound work)
#         return f"processed_{item}"
#     
#     # Create concurrent tasks
#     tasks = [asyncio.create_task(process_item(item)) for item in items]
#     
#     # Wait for all tasks to complete
#     results = await asyncio.gather(*tasks)
#     
#     return {"items": items, "results": results, "count": len(results)}
#     
#     # Note: WASM limitations - asyncio.sleep() is not supported
#     # Async is useful for concurrent task coordination, not I/O delays


# Create the Spin handler
IncomingHandler = ftl.create_handler()