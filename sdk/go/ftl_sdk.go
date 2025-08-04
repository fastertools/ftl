// Package ftl provides a zero-dependency SDK for building MCP tools with Go.
//
// This SDK provides a thin layer over Spin Go SDK to implement the
// Model Context Protocol (MCP) for FTL tools.
package ftl

import (
	"fmt"
	"os"
	"strings"
)

// Utility functions for SDK

// isDebugEnabled checks if debug logging is enabled via environment variable
func isDebugEnabled() bool {
	return os.Getenv("FTL_DEBUG") == "true"
}

// secureLogf provides controlled debug logging without exposing sensitive data
func secureLogf(format string, args ...interface{}) {
	if isDebugEnabled() {
		fmt.Printf("[DEBUG] "+format+"\n", args...)
	}
}

// sanitizePath removes potentially sensitive query parameters from path logging
func sanitizePath(path string) string {
	if idx := strings.Index(path, "?"); idx != -1 {
		return path[:idx] + "?[REDACTED]"
	}
	return path
}

// camelToSnake converts camelCase to snake_case
func camelToSnake(s string) string {
	if s == "" {
		return ""
	}

	var result strings.Builder
	for i, r := range s {
		if i > 0 && r >= 'A' && r <= 'Z' {
			result.WriteRune('_')
		}
		if r >= 'A' && r <= 'Z' {
			result.WriteRune(r + ('a' - 'A'))
		} else {
			result.WriteRune(r)
		}
	}
	return result.String()
}

// Content type constants
const (
	ContentTypeText     = "text"
	ContentTypeImage    = "image"
	ContentTypeAudio    = "audio"
	ContentTypeResource = "resource"
)

// ToolMetadata represents tool metadata returned by GET requests
type ToolMetadata struct {
	// The name of the tool (must be unique within the gateway)
	Name string `json:"name"`

	// Optional human-readable title for the tool
	Title string `json:"title,omitempty"`

	// Optional description of what the tool does
	Description string `json:"description,omitempty"`

	// JSON Schema describing the expected input parameters
	InputSchema map[string]interface{} `json:"inputSchema"`

	// Optional JSON Schema describing the output format
	OutputSchema map[string]interface{} `json:"outputSchema,omitempty"`

	// Optional annotations providing hints about tool behavior
	Annotations *ToolAnnotations `json:"annotations,omitempty"`

	// Optional metadata for tool-specific extensions
	Meta map[string]interface{} `json:"_meta,omitempty"`
}

// ToolAnnotations provides hints about tool behavior
type ToolAnnotations struct {
	// Optional title annotation
	Title string `json:"title,omitempty"`

	// Hint that the tool is read-only (doesn't modify state)
	ReadOnlyHint bool `json:"readOnlyHint,omitempty"`

	// Hint that the tool may perform destructive operations
	DestructiveHint bool `json:"destructiveHint,omitempty"`

	// Hint that the tool is idempotent (same input → same output)
	IdempotentHint bool `json:"idempotentHint,omitempty"`

	// Hint that the tool accepts open-world inputs
	OpenWorldHint bool `json:"openWorldHint,omitempty"`
}

// ToolResponse represents the response format for tool execution
type ToolResponse struct {
	// Array of content items returned by the tool
	Content []ToolContent `json:"content"`

	// Optional structured content matching the outputSchema
	StructuredContent interface{} `json:"structuredContent,omitempty"`

	// Indicates if this response represents an error
	IsError bool `json:"isError,omitempty"`
}

// ToolContent represents content that can be returned by tools
type ToolContent struct {
	// Content type discriminator
	Type string `json:"type"`

	// Text content (for type="text")
	Text string `json:"text,omitempty"`

	// Base64-encoded data (for type="image" or "audio")
	Data string `json:"data,omitempty"`

	// MIME type (for type="image" or "audio")
	MimeType string `json:"mimeType,omitempty"`

	// Resource contents (for type="resource")
	Resource *ResourceContents `json:"resource,omitempty"`

	// Optional annotations for this content
	Annotations *ContentAnnotations `json:"annotations,omitempty"`
}

// ContentAnnotations provides metadata for content items
type ContentAnnotations struct {
	// Target audience for this content
	Audience []string `json:"audience,omitempty"`

	// Priority of this content (0.0 to 1.0)
	Priority float64 `json:"priority,omitempty"`
}

// ResourceContents represents resource data
type ResourceContents struct {
	// URI of the resource
	URI string `json:"uri"`

	// MIME type of the resource
	MimeType string `json:"mimeType,omitempty"`

	// Text content of the resource
	Text string `json:"text,omitempty"`

	// Base64-encoded binary content of the resource
	Blob string `json:"blob,omitempty"`
}

// ToolHandler is the function signature for tool handlers
type ToolHandler func(input map[string]interface{}) ToolResponse

// ToolDefinition defines a tool's configuration
type ToolDefinition struct {
	// Optional explicit tool name (overrides the map key)
	Name string

	// Optional human-readable title for the tool
	Title string

	// Optional description of what the tool does
	Description string

	// JSON Schema describing the expected input parameters
	InputSchema map[string]interface{}

	// Optional JSON Schema describing the output format
	OutputSchema map[string]interface{}

	// Optional annotations providing hints about tool behavior
	Annotations *ToolAnnotations

	// Optional metadata for tool-specific extensions
	Meta map[string]interface{}

	// Handler function for tool execution
	Handler ToolHandler
}

// Text creates a simple text response
func Text(text string) ToolResponse {
	return ToolResponse{
		Content: []ToolContent{
			{
				Type: "text",
				Text: text,
			},
		},
	}
}

// Textf creates a formatted text response
func Textf(format string, args ...interface{}) ToolResponse {
	return Text(fmt.Sprintf(format, args...))
}

// Error creates an error response
func Error(err string) ToolResponse {
	return ToolResponse{
		Content: []ToolContent{
			{
				Type: "text",
				Text: err,
			},
		},
		IsError: true,
	}
}

// Errorf creates a formatted error response
func Errorf(format string, args ...interface{}) ToolResponse {
	return Error(fmt.Sprintf(format, args...))
}

// WithStructured creates a response with structured content
func WithStructured(text string, structured interface{}) ToolResponse {
	return ToolResponse{
		Content: []ToolContent{
			{
				Type: "text",
				Text: text,
			},
		},
		StructuredContent: structured,
	}
}

// TextContent creates a text content item
func TextContent(text string, annotations *ContentAnnotations) ToolContent {
	return ToolContent{
		Type:        "text",
		Text:        text,
		Annotations: annotations,
	}
}

// ImageContent creates an image content item
func ImageContent(data, mimeType string, annotations *ContentAnnotations) ToolContent {
	return ToolContent{
		Type:        "image",
		Data:        data,
		MimeType:    mimeType,
		Annotations: annotations,
	}
}

// AudioContent creates an audio content item
func AudioContent(data, mimeType string, annotations *ContentAnnotations) ToolContent {
	return ToolContent{
		Type:        "audio",
		Data:        data,
		MimeType:    mimeType,
		Annotations: annotations,
	}
}

// ResourceContent creates a resource content item
func ResourceContent(resource *ResourceContents, annotations *ContentAnnotations) ToolContent {
	return ToolContent{
		Type:        "resource",
		Resource:    resource,
		Annotations: annotations,
	}
}

// CreateTools creates a Spin HTTP handler for MCP tools.
//
// Example:
//
//	func init() {
//	    CreateTools(map[string]ToolDefinition{
//	        "echo": {
//	            Description: "Echo the input",
//	            InputSchema: map[string]interface{}{
//	                "type": "object",
//	                "properties": map[string]interface{}{
//	                    "message": map[string]interface{}{
//	                        "type": "string",
//	                        "description": "The message to echo",
//	                    },
//	                },
//	                "required": []string{"message"},
//	            },
//	            Handler: func(input map[string]interface{}) ToolResponse {
//	                message, _ := input["message"].(string)
//	                return Text(fmt.Sprintf("Echo: %s", message))
//	            },
//	        },
//	    })
//	}
//
//	func main() {}

// Type guards for content types

// IsTextContent checks if content is text type
func IsTextContent(c *ToolContent) bool {
	return c.Type == ContentTypeText
}

// IsImageContent checks if content is image type
func IsImageContent(c *ToolContent) bool {
	return c.Type == ContentTypeImage
}

// IsAudioContent checks if content is audio type
func IsAudioContent(c *ToolContent) bool {
	return c.Type == ContentTypeAudio
}

// IsResourceContent checks if content is resource type
func IsResourceContent(c *ToolContent) bool {
	return c.Type == ContentTypeResource
}
