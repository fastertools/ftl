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
	// Validate tools input to prevent runtime issues
	if tools == nil {
		tools = make(map[string]ToolDefinition)
	}

	// Capture tools in closure with validation
	toolsCopy := make(map[string]ToolDefinition)
	for k, v := range tools {
		// Skip invalid entries to prevent runtime issues
		if k == "" {
			continue
		}
		toolsCopy[k] = v
	}

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

		// Handle GET / - return tool metadata
		if method == "GET" && (path == "/" || path == "") {
			secureLogf("Handling GET request for tools metadata, found %d tools", len(toolsCopy))
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
			if err := json.NewEncoder(w).Encode(metadata); err != nil {
				safeWriteError(w, "Failed to encode response", http.StatusInternalServerError)
				return
			}
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
				return
			}
			return
		}

		// Method not allowed
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
	})
}
