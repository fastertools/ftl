"""FTL SDK for Python - Build MCP tools that compile to WebAssembly."""

# Core API
from .ftl import FTL

# Response and content helpers
from .response import (
    ToolContent,
    ToolResponse,
    ToolResult,
    is_audio_content,
    is_image_content,
    is_resource_content,
    is_text_content,
)

__version__ = "0.1.0"

__all__ = [
    # Core API
    "FTL",
    # Response helpers
    "ToolResponse",
    "ToolResult",
    "ToolContent",
    # Content type guards
    "is_text_content",
    "is_image_content",
    "is_audio_content",
    "is_resource_content",
]
