"""
FTL SDK for Python - Zero-dependency SDK for building MCP tools.

This SDK provides a thin layer over Spin Python SDK to implement the
Model Context Protocol (MCP) for FTL tools.
"""

from collections.abc import Callable
from typing import Any

# Type aliases for clarity
ToolHandler = Callable[[dict[str, Any]], dict[str, Any]]
JSONSchema = dict[str, Any]


class ToolResponse:
    """Helper class for creating MCP-compliant tool responses."""

    @staticmethod
    def text(text: str) -> dict[str, Any]:
        """Create a simple text response."""
        return {"content": [{"type": "text", "text": text}]}

    @staticmethod
    def error(error: str) -> dict[str, Any]:
        """Create an error response."""
        return {"content": [{"type": "text", "text": error}], "isError": True}

    @staticmethod
    def with_structured(text: str, structured: Any) -> dict[str, Any]:
        """Create a response with structured content."""
        return {"content": [{"type": "text", "text": text}], "structuredContent": structured}


class ToolResult:
    """
    FastMCP-style tool result with simple constructor API.

    Examples:
        # Simple text content
        return ToolResult("Hello world")

        # Structured content only
        return ToolResult(structured_content={"status": "success", "count": 42})

        # Both content and structured content
        return ToolResult("Process completed", {"status": "success"})

        # List of content blocks
        return ToolResult([
            {"type": "text", "text": "Result"},
            {"type": "image", "data": "base64...", "mimeType": "image/png"}
        ])
    """

    def __init__(
        self,
        content: str | list[dict[str, Any]] | dict[str, Any] | None = None,
        structured_content: Any | None = None,
    ):
        """
        Initialize a ToolResult with content and/or structured content.

        Args:
            content: Content for the response. Can be:
                    - str: Will be converted to text content
                    - List[Dict]: List of MCP content blocks
                    - Dict: Single MCP content block
                    - None: No content (structured_content must be provided)
            structured_content: Optional structured data for the response

        Raises:
            ValueError: If both content and structured_content are None
        """
        if content is None and structured_content is None:
            raise ValueError("Either content or structured_content must be provided")

        # Always process content through _convert_to_content
        # If content is None, pass structured_content directly to _convert_to_content
        if content is None:
            # Pass structured_content directly - it will be wrapped appropriately
            content_to_convert = structured_content
        else:
            content_to_convert = content

        self.content = self._convert_to_content(content_to_convert)
        self.structured_content = structured_content

    def _convert_to_content(self, content: str | list[dict[str, Any]] | dict[str, Any] | Any) -> list[dict[str, Any]]:
        """
        Convert various content types to MCP content block list.

        Args:
            content: Content to convert

        Returns:
            List of MCP content blocks
        """
        if isinstance(content, str):
            # String -> text content block
            return [ToolContent.text(content)]
        elif isinstance(content, list):
            # List -> assume it's already a list of content blocks
            return content
        elif isinstance(content, dict):
            # Dict -> assume it's a single content block
            return [content]
        else:
            # Fallback: convert to string
            return [ToolContent.text(str(content))]

    def to_mcp_result(self) -> list[dict[str, Any]] | tuple[list[dict[str, Any]], dict[str, Any]]:
        """
        Convert to MCP result format (FastMCP compatibility).

        Returns:
            Content blocks, or tuple of (content blocks, structured content)
        """
        if self.structured_content is None:
            return self.content
        return self.content, self.structured_content


class ToolContent:
    """Helper class for creating different types of content."""

    @staticmethod
    def text(text: str, annotations: dict[str, Any] | None = None) -> dict[str, Any]:
        """Create text content."""
        content: dict[str, Any] = {"type": "text", "text": text}
        if annotations:
            content["annotations"] = annotations
        return content

    @staticmethod
    def image(data: str, mime_type: str, annotations: dict[str, Any] | None = None) -> dict[str, Any]:
        """Create image content."""
        content: dict[str, Any] = {"type": "image", "data": data, "mimeType": mime_type}
        if annotations:
            content["annotations"] = annotations
        return content

    @staticmethod
    def audio(data: str, mime_type: str, annotations: dict[str, Any] | None = None) -> dict[str, Any]:
        """Create audio content."""
        content: dict[str, Any] = {"type": "audio", "data": data, "mimeType": mime_type}
        if annotations:
            content["annotations"] = annotations
        return content

    @staticmethod
    def resource(resource: dict[str, Any], annotations: dict[str, Any] | None = None) -> dict[str, Any]:
        """Create resource content."""
        content = {"type": "resource", "resource": resource}
        if annotations:
            content["annotations"] = annotations
        return content


def _camel_to_snake(name: str) -> str:
    """Convert camelCase to snake_case."""
    result: list[str] = []
    for i, char in enumerate(name):
        if char.isupper() and i > 0:
            result.append("_")
        result.append(char.lower())
    return "".join(result)


# Type guards for content types
def is_text_content(content: dict[str, Any]) -> bool:
    """Check if content is text type."""
    return content.get("type") == "text"


def is_image_content(content: dict[str, Any]) -> bool:
    """Check if content is image type."""
    return content.get("type") == "image"


def is_audio_content(content: dict[str, Any]) -> bool:
    """Check if content is audio type."""
    return content.get("type") == "audio"


def is_resource_content(content: dict[str, Any]) -> bool:
    """Check if content is resource type."""
    return content.get("type") == "resource"
