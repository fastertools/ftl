"""FTL SDK for Python - Build MCP tools that compile to WebAssembly."""

# Python 3.10 compatibility shim for spin-sdk
import sys

if sys.version_info < (3, 11):
    try:
        from typing import Self  # type: ignore[attr-defined]
    except ImportError:
        import typing

        from typing_extensions import Self

        typing.Self = Self  # type: ignore[attr-defined]

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

__version__ = "0.10.1"

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
