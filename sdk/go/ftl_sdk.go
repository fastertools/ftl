// Package ftl provides a zero-dependency SDK for building MCP tools with Go.
//
// This SDK provides a thin layer over Spin Go SDK to implement the
// Model Context Protocol (MCP) for FTL tools.
package ftl

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	spinhttp "github.com/fermyon/spin/sdk/go/v2/http"
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

// camelToSnake converts camelCase to snake_case
func camelToSnake(s string) string {
	var result strings.Builder
	for i, r := range s {
		if i > 0 && r >= 'A' && r <= 'Z' {
			result.WriteRune('_')
		}
		result.WriteRune(r)
	}
	return strings.ToLower(result.String())
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
func CreateTools(tools map[string]ToolDefinition) {
	// Capture tools in closure
	toolsCopy := make(map[string]ToolDefinition)
	for k, v := range tools {
		toolsCopy[k] = v
	}
	
	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		path := r.URL.Path
		method := r.Method

		// Log request for debugging
		fmt.Printf("[DEBUG] Method: %s, Path: '%s', URI: '%s', Tools count: %d\n", method, path, r.RequestURI, len(toolsCopy))
		
		// Debug: Print tool names
		for key := range toolsCopy {
			fmt.Printf("[DEBUG] Tool key: %s\n", key)
		}

		// Handle GET / - return tool metadata
		if method == "GET" && (path == "/" || path == "") {
			fmt.Printf("[DEBUG] Handling GET request for tools metadata, found %d tools\n", len(toolsCopy))
			metadata := make([]ToolMetadata, 0, len(toolsCopy))
			for key, tool := range toolsCopy {
				// Use explicit name if provided, otherwise convert from key
				toolName := tool.Name
				if toolName == "" {
					toolName = camelToSnake(key)
				}

				// Set default input schema if not provided
				inputSchema := tool.InputSchema
				if inputSchema == nil {
					inputSchema = map[string]interface{}{"type": "object"}
				}

				metadata = append(metadata, ToolMetadata{
					Name:         toolName,
					Title:        tool.Title,
					Description:  tool.Description,
					InputSchema:  inputSchema,
					OutputSchema: tool.OutputSchema,
					Annotations:  tool.Annotations,
					Meta:         tool.Meta,
				})
			}

			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(metadata)
			return
		}

		// Handle POST /{tool_name} - execute tool
		if method == "POST" && len(path) > 1 {
			toolName := strings.TrimPrefix(path, "/")

			// Find the tool by name
			var toolEntry *ToolDefinition
			for key, tool := range toolsCopy {
				effectiveName := tool.Name
				if effectiveName == "" {
					effectiveName = camelToSnake(key)
				}
				if effectiveName == toolName {
					toolEntry = &tool
					break
				}
			}

			if toolEntry == nil {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(404)
				json.NewEncoder(w).Encode(Error(fmt.Sprintf("Tool '%s' not found", toolName)))
				return
			}

			// Parse input
			var input map[string]interface{}
			if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
				// Handle empty body
				input = make(map[string]interface{})
			}

			// Execute handler
			result := toolEntry.Handler(input)

			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(result)
			return
		}

		// Method not allowed
		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Allow", "GET, POST")
		w.WriteHeader(405)
		json.NewEncoder(w).Encode(map[string]interface{}{
			"error": map[string]interface{}{
				"code":    -32601,
				"message": "Method not allowed",
			},
		})
	})
}

// Type guards for content types

// IsTextContent checks if content is text type
func IsTextContent(c ToolContent) bool {
	return c.Type == "text"
}

// IsImageContent checks if content is image type
func IsImageContent(c ToolContent) bool {
	return c.Type == "image"
}

// IsAudioContent checks if content is audio type
func IsAudioContent(c ToolContent) bool {
	return c.Type == "audio"
}

// IsResourceContent checks if content is resource type
func IsResourceContent(c ToolContent) bool {
	return c.Type == "resource"
}