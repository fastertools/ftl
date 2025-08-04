package ftl

import (
	"testing"
)

// Test utility functions

func TestIsDebugEnabled(t *testing.T) {
	// Test with debug disabled (default)
	t.Setenv("FTL_DEBUG", "")
	if isDebugEnabled() {
		t.Error("Expected isDebugEnabled() to return false when FTL_DEBUG is empty")
	}

	// Test with debug enabled
	t.Setenv("FTL_DEBUG", "true")
	if !isDebugEnabled() {
		t.Error("Expected isDebugEnabled() to return true when FTL_DEBUG=true")
	}

	// Test with invalid value
	t.Setenv("FTL_DEBUG", "invalid")
	if isDebugEnabled() {
		t.Error("Expected isDebugEnabled() to return false when FTL_DEBUG=invalid")
	}
}

func TestSanitizePath(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"/api/user?token=secret123", "/api/user?[REDACTED]"},
		{"/api/user?param1=value1&token=secret", "/api/user?[REDACTED]"},
		{"/api/user", "/api/user"},
		{"", ""},
		{"just-a-string", "just-a-string"},
		{"path?multiple=params&secret=hidden", "path?[REDACTED]"},
	}

	for _, test := range tests {
		result := sanitizePath(test.input)
		if result != test.expected {
			t.Errorf("sanitizePath(%q) = %q; want %q", test.input, result, test.expected)
		}
	}
}

func TestSecureLog(t *testing.T) {
	// This test ensures secureLog doesn't panic and respects debug flag
	// We can't easily test the actual output without capturing stdout

	// Test with debug disabled
	t.Setenv("FTL_DEBUG", "")
	// Should not panic
	secureLogf("test message: %s", "value")

	// Test with debug enabled
	t.Setenv("FTL_DEBUG", "true")
	// Should not panic
	secureLogf("test message: %s", "value")
}

func TestCamelToSnake(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"camelCase", "camel_case"},
		{"PascalCase", "pascal_case"},
		{"lowercase", "lowercase"},
		{"UPPERCASE", "u_p_p_e_r_c_a_s_e"},
		{"mixedUPPERCase", "mixed_u_p_p_e_r_case"},
		{"", ""},
	}

	for _, test := range tests {
		result := camelToSnake(test.input)
		if result != test.expected {
			t.Errorf("camelToSnake(%q) = %q; want %q", test.input, result, test.expected)
		}
	}
}

// Test response creation functions

func TestText(t *testing.T) {
	resp := Text("Hello, world!")

	if len(resp.Content) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(resp.Content))
	}

	if resp.Content[0].Type != "text" {
		t.Errorf("Expected content type 'text', got %q", resp.Content[0].Type)
	}

	if resp.Content[0].Text != "Hello, world!" {
		t.Errorf("Expected text 'Hello, world!', got %q", resp.Content[0].Text)
	}

	if resp.IsError {
		t.Error("Expected IsError to be false")
	}
}

func TestTextf(t *testing.T) {
	resp := Textf("Hello, %s! You are %d years old.", "Alice", 30)
	expected := "Hello, Alice! You are 30 years old."

	if resp.Content[0].Text != expected {
		t.Errorf("Expected text %q, got %q", expected, resp.Content[0].Text)
	}
}

func TestError(t *testing.T) {
	resp := Error("Something went wrong")

	if !resp.IsError {
		t.Error("Expected IsError to be true")
	}

	if resp.Content[0].Text != "Something went wrong" {
		t.Errorf("Expected error text 'Something went wrong', got %q", resp.Content[0].Text)
	}
}

func TestErrorf(t *testing.T) {
	resp := Errorf("Error code: %d", 404)
	expected := "Error code: 404"

	if resp.Content[0].Text != expected {
		t.Errorf("Expected text %q, got %q", expected, resp.Content[0].Text)
	}

	if !resp.IsError {
		t.Error("Expected IsError to be true")
	}
}

func TestWithStructured(t *testing.T) {
	data := map[string]interface{}{
		"result": 42,
		"status": "success",
	}

	resp := WithStructured("Operation completed", data)

	if resp.Content[0].Text != "Operation completed" {
		t.Errorf("Expected text 'Operation completed', got %q", resp.Content[0].Text)
	}

	structured, ok := resp.StructuredContent.(map[string]interface{})
	if !ok {
		t.Fatal("Expected StructuredContent to be map[string]interface{}")
	}

	if structured["result"] != 42 {
		t.Errorf("Expected result 42, got %v", structured["result"])
	}
}

// Test content creation functions

func TestContentCreators(t *testing.T) {
	// Test TextContent
	textContent := TextContent("Hello", nil)
	if textContent.Type != "text" || textContent.Text != "Hello" {
		t.Error("TextContent not created correctly")
	}

	// Test ImageContent
	imageContent := ImageContent("base64data", "image/png", nil)
	if imageContent.Type != "image" || imageContent.Data != "base64data" || imageContent.MimeType != "image/png" {
		t.Error("ImageContent not created correctly")
	}

	// Test AudioContent
	audioContent := AudioContent("base64audio", "audio/wav", nil)
	if audioContent.Type != "audio" || audioContent.Data != "base64audio" || audioContent.MimeType != "audio/wav" {
		t.Error("AudioContent not created correctly")
	}

	// Test ResourceContent
	resource := &ResourceContents{
		URI:      "file://test.txt",
		MimeType: "text/plain",
		Text:     "content",
	}
	resourceContent := ResourceContent(resource, nil)
	if resourceContent.Type != "resource" || resourceContent.Resource.URI != "file://test.txt" {
		t.Error("ResourceContent not created correctly")
	}
}

// Test type guards

func TestContentTypeGuards(t *testing.T) {
	textContent := ToolContent{Type: ContentTypeText, Text: "Hello"}
	imageContent := ToolContent{Type: ContentTypeImage, Data: "data", MimeType: "image/png"}
	audioContent := ToolContent{Type: ContentTypeAudio, Data: "data", MimeType: "audio/wav"}
	resourceContent := ToolContent{Type: ContentTypeResource, Resource: &ResourceContents{URI: "test"}}

	if !IsTextContent(&textContent) {
		t.Error("IsTextContent failed for text content")
	}

	if !IsImageContent(&imageContent) {
		t.Error("IsImageContent failed for image content")
	}

	if !IsAudioContent(&audioContent) {
		t.Error("IsAudioContent failed for audio content")
	}

	if !IsResourceContent(&resourceContent) {
		t.Error("IsResourceContent failed for resource content")
	}

	// Test negative cases
	if IsTextContent(&imageContent) {
		t.Error("IsTextContent returned true for image content")
	}
}

// Test tool definitions (structure validation without HTTP)

func TestToolDefinitionStructure(t *testing.T) {
	// Test that we can create valid tool definitions without HTTP context
	toolDef := ToolDefinition{
		Description: "Test tool",
		InputSchema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"input": map[string]interface{}{"type": "string"},
			},
		},
		Handler: func(_ map[string]interface{}) ToolResponse {
			return Text("test response")
		},
	}

	if toolDef.Description != "Test tool" {
		t.Error("Tool description not set correctly")
	}

	if toolDef.Handler == nil {
		t.Error("Tool handler not set")
	}

	// Test handler function
	testInput := map[string]interface{}{"input": "test"}
	response := toolDef.Handler(testInput)

	if response.IsError {
		t.Error("Expected successful response from test handler")
	}

	if len(response.Content) == 0 || response.Content[0].Text != "test response" {
		t.Error("Handler didn't return expected response")
	}
}
