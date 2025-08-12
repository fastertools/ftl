//go:build !test

package ftl

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	spinhttp "github.com/spinframework/spin-go-sdk/http"
)

// safeWriteError writes an error response with proper headers and status
func safeWriteError(w http.ResponseWriter, message string, statusCode int) {
	// Ensure headers are set before writing status
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(statusCode)

	errorResponse := map[string]interface{}{
		"error": message,
		"code":  statusCode,
	}

	// Use encoder to prevent JSON marshaling panics
	encoder := json.NewEncoder(w)
	if err := encoder.Encode(errorResponse); err != nil {
		// Fallback to plain text if JSON encoding fails
		w.Header().Set("Content-Type", "text/plain")
		fmt.Fprintf(w, "Internal Server Error: %d", statusCode)
	}
}

// validateAndCopyTools validates the input tools map and returns a clean copy
func validateAndCopyTools(tools map[string]ToolDefinition) map[string]ToolDefinition {
	if tools == nil {
		return make(map[string]ToolDefinition)
	}

	toolsCopy := make(map[string]ToolDefinition)
	for k, v := range tools {
		// Skip invalid entries to prevent runtime issues
		if k == "" {
			continue
		}
		toolsCopy[k] = v
	}
	return toolsCopy
}

// buildToolMetadata generates metadata for all registered tools
func buildToolMetadata(tools map[string]ToolDefinition) []ToolMetadata {
	metadata := make([]ToolMetadata, 0, len(tools))
	for key, tool := range tools {
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
	return metadata
}

// findToolByName searches for a tool by its name in the tools map
func findToolByName(tools map[string]ToolDefinition, toolName string) *ToolDefinition {
	for key, tool := range tools {
		effectiveName := tool.Name
		if effectiveName == "" {
			effectiveName = camelToSnake(key)
		}
		if effectiveName == toolName {
			return &tool
		}
	}
	return nil
}

// handleGetToolsMetadata handles GET / requests for tool metadata
func handleGetToolsMetadata(w http.ResponseWriter, tools map[string]ToolDefinition) {
	secureLogf("Handling GET request for tools metadata, found %d tools", len(tools))
	metadata := buildToolMetadata(tools)

	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(metadata); err != nil {
		safeWriteError(w, "Failed to encode response", http.StatusInternalServerError)
	}
}

// handlePostToolExecution handles POST /{tool_name} requests for tool execution
func handlePostToolExecution(w http.ResponseWriter, r *http.Request, tools map[string]ToolDefinition) {
	toolName := strings.TrimPrefix(r.URL.Path, "/")

	toolEntry := findToolByName(tools, toolName)
	if toolEntry == nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(404)
		if err := json.NewEncoder(w).Encode(Error(fmt.Sprintf("Tool '%s' not found", toolName))); err != nil {
			safeWriteError(w, "Tool not found", http.StatusNotFound)
		}
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
	if err := json.NewEncoder(w).Encode(result); err != nil {
		safeWriteError(w, "Failed to encode tool result", http.StatusInternalServerError)
	}
}

// handleMethodNotAllowed handles unsupported HTTP methods
func handleMethodNotAllowed(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Allow", "GET, POST")
	w.WriteHeader(405)
	if err := json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]interface{}{
			"code":    -32601,
			"message": "Method not allowed",
		},
	}); err != nil {
		safeWriteError(w, "Method not allowed", http.StatusMethodNotAllowed)
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
func CreateTools(tools map[string]ToolDefinition) {
	toolsCopy := validateAndCopyTools(tools)

	spinhttp.Handle(func(w http.ResponseWriter, r *http.Request) {
		// Defensive programming: validate request before processing
		if r == nil {
			safeWriteError(w, "Invalid request", http.StatusBadRequest)
			return
		}

		path := r.URL.Path
		method := r.Method

		// Secure logging for debugging (only logs when FTL_DEBUG=true)
		secureLogf("Method: %s, Path: '%s', Tools count: %d", method, sanitizePath(path), len(toolsCopy))

		// Debug: Log tool count only (tool names could be sensitive)
		if isDebugEnabled() {
			secureLogf("Available tools: %d registered", len(toolsCopy))
		}

		// Route requests to appropriate handlers
		switch {
		case method == "GET" && (path == "/" || path == ""):
			handleGetToolsMetadata(w, toolsCopy)
		case method == "POST" && len(path) > 1:
			handlePostToolExecution(w, r, toolsCopy)
		default:
			handleMethodNotAllowed(w)
		}
	})
}
