package ftl

import (
	"encoding/json"
	"testing"
)

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

func TestTextResponse(t *testing.T) {
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

func TestTextfResponse(t *testing.T) {
	resp := Textf("Hello, %s! You are %d years old.", "Alice", 30)
	expected := "Hello, Alice! You are 30 years old."
	
	if resp.Content[0].Text != expected {
		t.Errorf("Expected text %q, got %q", expected, resp.Content[0].Text)
	}
}

func TestErrorResponse(t *testing.T) {
	resp := Error("Something went wrong")
	
	if !resp.IsError {
		t.Error("Expected IsError to be true")
	}
	
	if resp.Content[0].Text != "Something went wrong" {
		t.Errorf("Expected error text 'Something went wrong', got %q", resp.Content[0].Text)
	}
}

func TestErrorfResponse(t *testing.T) {
	resp := Errorf("Error code: %d", 404)
	expected := "Error code: 404"
	
	if resp.Content[0].Text != expected {
		t.Errorf("Expected text %q, got %q", expected, resp.Content[0].Text)
	}
	
	if !resp.IsError {
		t.Error("Expected IsError to be true")
	}
}

func TestWithStructuredResponse(t *testing.T) {
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

func TestContentTypeGuards(t *testing.T) {
	textContent := ToolContent{Type: "text", Text: "Hello"}
	imageContent := ToolContent{Type: "image", Data: "data", MimeType: "image/png"}
	audioContent := ToolContent{Type: "audio", Data: "data", MimeType: "audio/wav"}
	resourceContent := ToolContent{Type: "resource", Resource: &ResourceContents{URI: "test"}}
	
	if !IsTextContent(textContent) {
		t.Error("IsTextContent failed for text content")
	}
	
	if !IsImageContent(imageContent) {
		t.Error("IsImageContent failed for image content")
	}
	
	if !IsAudioContent(audioContent) {
		t.Error("IsAudioContent failed for audio content")
	}
	
	if !IsResourceContent(resourceContent) {
		t.Error("IsResourceContent failed for resource content")
	}
	
	// Test negative cases
	if IsTextContent(imageContent) {
		t.Error("IsTextContent returned true for image content")
	}
}

func TestToolMetadataJSON(t *testing.T) {
	metadata := ToolMetadata{
		Name:        "test_tool",
		Title:       "Test Tool",
		Description: "A test tool",
		InputSchema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"input": map[string]interface{}{
					"type": "string",
				},
			},
		},
		Annotations: &ToolAnnotations{
			ReadOnlyHint: true,
		},
	}
	
	// Test marshaling
	data, err := json.Marshal(metadata)
	if err != nil {
		t.Fatalf("Failed to marshal metadata: %v", err)
	}
	
	// Test unmarshaling
	var unmarshaled ToolMetadata
	if err := json.Unmarshal(data, &unmarshaled); err != nil {
		t.Fatalf("Failed to unmarshal metadata: %v", err)
	}
	
	if unmarshaled.Name != metadata.Name {
		t.Errorf("Expected name %q, got %q", metadata.Name, unmarshaled.Name)
	}
	
	if unmarshaled.Annotations == nil || !unmarshaled.Annotations.ReadOnlyHint {
		t.Error("Annotations not preserved correctly")
	}
}