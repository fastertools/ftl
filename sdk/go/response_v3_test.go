package ftl

import (
	"encoding/base64"
	"testing"
)

// TestNewResponseBuilder tests basic response builder creation
func TestNewResponseBuilder(t *testing.T) {
	rb := NewResponse()
	
	if rb == nil {
		t.Fatal("NewResponseBuilder should not return nil")
	}
	
	if rb.isError {
		t.Error("New response builder should not be in error state")
	}
	
	if len(rb.contents) != 0 {
		t.Error("New response builder should have empty contents")
	}
	
	if rb.structured != nil {
		t.Error("New response builder should have nil structured content")
	}
}

// TestResponseBuilder_AddText tests text content addition
func TestResponseBuilder_AddText(t *testing.T) {
	rb := NewResponse()
	
	// Test single text addition
	result := rb.AddText("Hello, World!")
	
	// Should return self for chaining
	if result != rb {
		t.Error("AddText should return self for method chaining")
	}
	
	if len(rb.contents) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(rb.contents))
	}
	
	content := rb.contents[0]
	if content.Type != "text" {
		t.Errorf("Expected content type 'text', got '%s'", content.Type)
	}
	
	if content.Text != "Hello, World!" {
		t.Errorf("Expected text 'Hello, World!', got '%s'", content.Text)
	}
	
	// Test multiple text additions
	rb.AddText("Second message")
	
	if len(rb.contents) != 2 {
		t.Errorf("Expected 2 content items, got %d", len(rb.contents))
	}
	
	// Test empty text (should still be added)
	rb.AddText("")
	
	if len(rb.contents) != 3 {
		t.Errorf("Expected 3 content items, got %d", len(rb.contents))
	}
	
	if rb.contents[2].Text != "" {
		t.Error("Empty text should be preserved")
	}
}

// TestResponseBuilder_AddImage tests image content addition
func TestResponseBuilder_AddImage(t *testing.T) {
	rb := NewResponse()
	
	imageData := []byte("fake-image-data")
	result := rb.AddImage(imageData, "image/png")
	
	if result != rb {
		t.Error("AddImage should return self for method chaining")
	}
	
	if len(rb.contents) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(rb.contents))
	}
	
	content := rb.contents[0]
	if content.Type != "image" {
		t.Errorf("Expected content type 'image', got '%s'", content.Type)
	}
	
	if content.MimeType != "image/png" {
		t.Errorf("Expected mime type 'image/png', got '%s'", content.MimeType)
	}
	
	// Data should be base64 encoded string
	expectedEncoded := base64.StdEncoding.EncodeToString(imageData)
	if content.Data != expectedEncoded {
		t.Errorf("Expected base64 data %q, got %q", expectedEncoded, content.Data)
	}
	
	// Test with empty data
	rb.AddImage([]byte{}, "image/jpeg")
	
	if len(rb.contents) != 2 {
		t.Errorf("Expected 2 content items, got %d", len(rb.contents))
	}
	
	if len(rb.contents[1].Data) != 0 {
		t.Error("Empty image data should be preserved")
	}
}

// TestResponseBuilder_AddAudio tests audio content addition
func TestResponseBuilder_AddAudio(t *testing.T) {
	rb := NewResponse()
	
	audioData := []byte("fake-audio-data")
	result := rb.AddAudio(audioData, "audio/wav")
	
	if result != rb {
		t.Error("AddAudio should return self for method chaining")
	}
	
	if len(rb.contents) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(rb.contents))
	}
	
	content := rb.contents[0]
	if content.Type != "audio" {
		t.Errorf("Expected content type 'audio', got '%s'", content.Type)
	}
	
	if content.MimeType != "audio/wav" {
		t.Errorf("Expected mime type 'audio/wav', got '%s'", content.MimeType)
	}
	
	// Data should be base64 encoded string
	expectedEncoded := base64.StdEncoding.EncodeToString(audioData)
	if content.Data != expectedEncoded {
		t.Errorf("Expected base64 data %q, got %q", expectedEncoded, content.Data)
	}
}

// TestResponseBuilder_AddResource tests resource content addition
func TestResponseBuilder_AddResource(t *testing.T) {
	rb := NewResponse()
	
	resource := &ResourceContents{
		URI:      "https://example.com/resource",
		MimeType: "text/plain",
		Text:     "A test resource",
	}
	result := rb.AddResource(resource)
	
	if result != rb {
		t.Error("AddResource should return self for method chaining")
	}
	
	if len(rb.contents) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(rb.contents))
	}
	
	content := rb.contents[0]
	if content.Type != "resource" {
		t.Errorf("Expected content type 'resource', got '%s'", content.Type)
	}
	
	if content.Resource == nil {
		t.Error("Expected resource content to be set")
	} else {
		if content.Resource.URI != "https://example.com/resource" {
			t.Errorf("Expected URI 'https://example.com/resource', got '%s'", content.Resource.URI)
		}
		if content.Resource.Text != "A test resource" {
			t.Errorf("Expected text 'A test resource', got '%s'", content.Resource.Text)
		}
	}
	
	// Test with another resource
	anotherResource := &ResourceContents{
		URI: "https://example.com/other",
	}
	rb.AddResource(anotherResource)
	
	if len(rb.contents) != 2 {
		t.Errorf("Expected 2 content items, got %d", len(rb.contents))
	}
	
	content2 := rb.contents[1]
	if content2.Resource == nil || content2.Resource.URI != "https://example.com/other" {
		t.Error("Second resource should be added correctly")
	}
}

// TestResponseBuilder_AddStructured tests structured content setting
func TestResponseBuilder_AddStructured(t *testing.T) {
	rb := NewResponse()
	
	data := map[string]interface{}{
		"status": "success",
		"count":  42,
		"items":  []string{"a", "b", "c"},
	}
	
	result := rb.AddStructured(data)
	
	if result != rb {
		t.Error("AddStructured should return self for method chaining")
	}
	
	if rb.structured == nil {
		t.Fatal("Structured data should not be nil")
	}
	
	structuredMap, ok := rb.structured.(map[string]interface{})
	if !ok {
		t.Fatal("Structured data should be a map")
	}
	
	if structuredMap["status"] != "success" {
		t.Errorf("Expected status 'success', got %v", structuredMap["status"])
	}
	
	// After JSON marshal/unmarshal, numbers become float64
	if count, ok := structuredMap["count"].(float64); !ok || count != 42.0 {
		t.Errorf("Expected count 42.0 (float64), got %v (%T)", structuredMap["count"], structuredMap["count"])
	}
	
	// Test overwriting structured data
	newData := map[string]interface{}{"new": "data"}
	rb.AddStructured(newData)
	
	structuredMap, ok = rb.structured.(map[string]interface{})
	if !ok {
		t.Fatal("New structured data should be a map")
	}
	
	if len(structuredMap) != 1 || structuredMap["new"] != "data" {
		t.Error("Structured data should be replaced, not merged")
	}
}

// TestResponseBuilder_WithError tests error response creation
func TestResponseBuilder_WithError(t *testing.T) {
	rb := NewResponse()
	rb.AddText("Error message")
	
	result := rb.WithError()
	
	if result != rb {
		t.Error("WithError should return self for method chaining")
	}
	
	if !rb.isError {
		t.Error("Response should be marked as error")
	}
	
	// Test that WithError can be called multiple times
	rb.WithError()
	if !rb.isError {
		t.Error("Response should remain as error")
	}
}

// TestResponseBuilder_Build tests response building
func TestResponseBuilder_Build(t *testing.T) {
	rb := NewResponse()
	rb.AddText("Hello")
	rb.AddText("World")
	rb.AddStructured(map[string]interface{}{"key": "value"})
	
	response := rb.Build()
	
	if len(response.Content) != 2 {
		t.Errorf("Expected 2 content items, got %d", len(response.Content))
	}
	
	if response.IsError {
		t.Error("Response should not be error")
	}
	
	if response.StructuredContent == nil {
		t.Error("Response should have structured data")
	}
	
	// Test error response building
	errorRb := NewResponse()
	errorRb.AddText("Error occurred")
	errorRb.WithError()
	
	errorResponse := errorRb.Build()
	
	if !errorResponse.IsError {
		t.Error("Error response should be marked as error")
	}
	
	if len(errorResponse.Content) != 1 {
		t.Errorf("Expected 1 error content item, got %d", len(errorResponse.Content))
	}
}

// TestResponseBuilder_Chaining tests method chaining
func TestResponseBuilder_Chaining(t *testing.T) {
	response := NewResponse().
		AddText("Start").
		AddText("Middle").
		AddStructured(map[string]interface{}{"chained": true}).
		AddText("End").
		Build()
	
	if len(response.Content) != 3 {
		t.Errorf("Expected 3 content items, got %d", len(response.Content))
	}
	
	if response.StructuredContent == nil {
		t.Error("Chained response should have structured data")
	}
	
	structured, ok := response.StructuredContent.(map[string]interface{})
	if !ok || structured["chained"] != true {
		t.Error("Structured data should contain chained: true")
	}
	
	// Test error chaining
	errorResponse := NewResponse().
		AddText("Error message").
		WithError().
		Build()
	
	if !errorResponse.IsError {
		t.Error("Chained error response should be marked as error")
	}
}

// TestResponseBuilder_MixedContent tests mixed content types
func TestResponseBuilder_MixedContent(t *testing.T) {
	rb := NewResponse()
	
	// Add various content types
	rb.AddText("Text content")
	rb.AddImage([]byte("image-data"), "image/png")
	rb.AddAudio([]byte("audio-data"), "audio/mp3")
	rb.AddResource(&ResourceContents{URI: "https://example.com", Text: "Example resource"})
	
	response := rb.Build()
	
	if len(response.Content) != 4 {
		t.Errorf("Expected 4 content items, got %d", len(response.Content))
	}
	
	// Verify content types in order
	expectedTypes := []string{"text", "image", "audio", "resource"}
	for i, expectedType := range expectedTypes {
		if response.Content[i].Type != expectedType {
			t.Errorf("Content item %d: expected type '%s', got '%s'", i, expectedType, response.Content[i].Type)
		}
	}
}

// TestResponseBuilder_EdgeCases tests edge cases and boundary conditions
func TestResponseBuilder_EdgeCases(t *testing.T) {
	// Test building empty response
	emptyResponse := NewResponse().Build()
	
	if len(emptyResponse.Content) != 0 {
		t.Errorf("Empty response should have 0 content items, got %d", len(emptyResponse.Content))
	}
	
	if emptyResponse.IsError {
		t.Error("Empty response should not be error")
	}
	
	if emptyResponse.StructuredContent != nil {
		t.Error("Empty response should not have structured data")
	}
	
	// Test very large content
	largeText := make([]byte, 1024*1024) // 1MB
	for i := range largeText {
		largeText[i] = 'A'
	}
	
	largeResponse := NewResponse().
		AddText(string(largeText)).
		Build()
	
	if len(largeResponse.Content) != 1 {
		t.Error("Large response should have 1 content item")
	}
	
	if len(largeResponse.Content[0].Text) != len(largeText) {
		t.Error("Large text should be preserved in full")
	}
	
	// Test nil structured data
	nilResponse := NewResponse().
		AddStructured(nil).
		Build()
	
	if nilResponse.StructuredContent != nil {
		t.Error("Nil structured data should remain nil")
	}
}

// TestResponseHelpers tests helper functions for common response patterns
func TestResponseHelpers(t *testing.T) {
	// Test TextResponse helper
	textResponse := TextResponse("Simple text")
	
	if len(textResponse.Content) != 1 {
		t.Errorf("TextResponse should have 1 content item, got %d", len(textResponse.Content))
	}
	
	if textResponse.Content[0].Type != "text" {
		t.Errorf("TextResponse content should be text type, got '%s'", textResponse.Content[0].Type)
	}
	
	if textResponse.Content[0].Text != "Simple text" {
		t.Errorf("TextResponse text should be 'Simple text', got '%s'", textResponse.Content[0].Text)
	}
	
	if textResponse.IsError {
		t.Error("TextResponse should not be error")
	}
	
	// Test ErrorResponse helper
	errorResponse := ErrorResponse("Something went wrong")
	
	if !errorResponse.IsError {
		t.Error("ErrorResponse should be marked as error")
	}
	
	if len(errorResponse.Content) != 1 {
		t.Errorf("ErrorResponse should have 1 content item, got %d", len(errorResponse.Content))
	}
	
	if errorResponse.Content[0].Text != "Something went wrong" {
		t.Errorf("ErrorResponse text should be 'Something went wrong', got '%s'", errorResponse.Content[0].Text)
	}
	
	// Test StructuredResponse helper
	data := map[string]interface{}{
		"result": "success",
		"data":   []int{1, 2, 3},
	}
	
	structuredResponse := StructuredResponse("Result data", data)
	
	if structuredResponse.StructuredContent == nil {
		t.Error("StructuredResponse should have structured data")
	}
	
	structuredMap, ok := structuredResponse.StructuredContent.(map[string]interface{})
	if !ok {
		t.Fatal("StructuredResponse should contain a map")
	}
	
	if structuredMap["result"] != "success" {
		t.Errorf("StructuredResponse should contain result: success, got %v", structuredMap["result"])
	}
}

// TestResponseBuilder_Immutability tests that responses are properly isolated
func TestResponseBuilder_Immutability(t *testing.T) {
	rb := NewResponse()
	rb.AddText("Original text")
	
	// Build first response
	response1 := rb.Build()
	
	// Modify builder and build second response
	rb.AddText("Additional text")
	response2 := rb.Build()
	
	// First response should be unchanged
	if len(response1.Content) != 1 {
		t.Errorf("First response should have 1 content item, got %d", len(response1.Content))
	}
	
	if response1.Content[0].Text != "Original text" {
		t.Error("First response text should not have changed")
	}
	
	// Second response should have both texts
	if len(response2.Content) != 2 {
		t.Errorf("Second response should have 2 content items, got %d", len(response2.Content))
	}
	
	// Test structured data isolation
	rb = NewResponse()
	originalData := map[string]interface{}{"shared": "data"}
	rb.AddStructured(originalData)
	
	response3 := rb.Build()
	
	// Modify original data
	originalData["shared"] = "modified"
	originalData["new"] = "key"
	
	// Response should not be affected
	responseData, ok := response3.StructuredContent.(map[string]interface{})
	if !ok {
		t.Fatal("Response structured data should be a map")
	}
	
	if responseData["shared"] != "data" {
		t.Error("Response structured data should not be affected by original data changes")
	}
	
	if _, exists := responseData["new"]; exists {
		t.Error("Response structured data should not have new keys from original")
	}
}