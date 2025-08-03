"""FTL SDK for Python - Build MCP tools that compile to WebAssembly."""

# Core API
from .ftl import FTL, tool

# Response and content helpers
from .response import (
    ToolResponse,
    ToolContent,
    is_text_content,
    is_image_content,
    is_audio_content,
    is_resource_content,
)

__version__ = "0.2.0"

__all__ = [
    # Core API
    "FTL",
    "tool",
    # Response helpers
    "ToolResponse",
    "ToolContent",
    # Content type guards
    "is_text_content",
    "is_image_content",
    "is_audio_content",
    "is_resource_content",
]