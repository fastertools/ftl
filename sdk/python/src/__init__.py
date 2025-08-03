"""FTL SDK for Python - Build MCP tools that compile to WebAssembly."""

from .ftl_sdk import (
    ToolContent,
    ToolResponse,
    create_tools,
    is_audio_content,
    is_image_content,
    is_resource_content,
    is_text_content,
)

__version__ = "0.1.0"

__all__ = [
    "create_tools",
    "ToolResponse",
    "ToolContent",
    "is_text_content",
    "is_image_content",
    "is_audio_content",
    "is_resource_content",
]
