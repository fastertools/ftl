package main

import (
	"testing"

	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

func TestExampleToolHandler(t *testing.T) {
	tests := []struct {
		name     string
		input    map[string]interface{}
		wantText string
		wantErr  bool
	}{
		{
			name: "valid message",
			input: map[string]interface{}{
				"message": "Hello, World!",
			},
			wantText: "Processed: Hello, World!",
			wantErr:  false,
		},
		{
			name:     "missing message",
			input:    map[string]interface{}{},
			wantText: "",
			wantErr:  true,
		},
		{
			name: "invalid message type",
			input: map[string]interface{}{
				"message": 123,
			},
			wantText: "",
			wantErr:  true,
		},
		{
			name: "empty message",
			input: map[string]interface{}{
				"message": "",
			},
			wantText: "Processed: ",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			response := exampleToolHandler(tt.input)
			
			// Check if response has content
			content, ok := response["content"].([]map[string]interface{})
			if !ok || len(content) == 0 {
				t.Fatal("Invalid response format: missing content")
			}
			
			// Check error state
			isError, _ := response["isError"].(bool)
			if isError != tt.wantErr {
				t.Errorf("Expected error state %v, got %v", tt.wantErr, isError)
			}
			
			// Check response text if not an error
			if !tt.wantErr {
				text, ok := content[0]["text"].(string)
				if !ok {
					t.Fatal("Response content should contain text")
				}
				if text != tt.wantText {
					t.Errorf("Expected text '%s', got '%s'", tt.wantText, text)
				}
			}
		})
	}
}

// Helper function to validate response structure
func validateToolResponse(t *testing.T, response ftl.ToolResponse) {
	t.Helper()
	
	// Check that response has content
	content, ok := response["content"].([]map[string]interface{})
	if !ok {
		t.Fatal("Response must have 'content' field as array")
	}
	
	if len(content) == 0 {
		t.Fatal("Response content must not be empty")
	}
	
	// Check first content item has type
	contentType, ok := content[0]["type"].(string)
	if !ok {
		t.Fatal("Content item must have 'type' field")
	}
	
	if contentType != "text" {
		t.Errorf("Expected content type 'text', got '%s'", contentType)
	}
}

// Example of testing with structured content
func TestStructuredResponse(t *testing.T) {
	// Example handler that returns structured content
	handler := func(input map[string]interface{}) ftl.ToolResponse {
		count, ok := input["count"].(float64) // JSON numbers are float64
		if !ok {
			return ftl.Error("count must be a number")
		}
		
		return ftl.WithStructured(
			"Operation complete",
			map[string]interface{}{
				"result": int(count) * 2,
				"original": int(count),
			},
		)
	}
	
	response := handler(map[string]interface{}{"count": 5.0})
	validateToolResponse(t, response)
	
	// Check structured content
	structured, ok := response["structuredContent"].(map[string]interface{})
	if !ok {
		t.Fatal("Expected structuredContent in response")
	}
	
	result, ok := structured["result"].(int)
	if !ok || result != 10 {
		t.Errorf("Expected result to be 10, got %v", structured["result"])
	}
}