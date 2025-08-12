package ftl

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"
)

// TestV3Integration verifies that the V3 API components work together
func TestV3Integration(t *testing.T) {
	// Test type definitions
	type TestInput struct {
		Message string `json:"message" jsonschema:"required,description=Test message"`
		Count   int    `json:"count,omitempty" jsonschema:"minimum=1,maximum=5"`
	}
	
	type TestOutput struct {
		Result string `json:"result"`
		Status string `json:"status"`
	}
	
	// Test handler definition
	handler := func(ctx context.Context, input TestInput) (TestOutput, error) {
		if input.Message == "" {
			return TestOutput{}, InvalidInput("message", "message is required")
		}
		
		result := input.Message
		if input.Count > 1 {
			for i := 1; i < input.Count; i++ {
				result += " " + input.Message
			}
		}
		
		return TestOutput{
			Result: result,
			Status: "success",
		}, nil
	}
	
	// Test handler registration (should not panic)
	HandleTypedTool("test_tool", handler)
	
	// Verify tool was registered
	if !IsV3Tool("test_tool") {
		t.Error("Tool was not registered as V3 tool")
	}
	
	// Check V3 tool names
	toolNames := GetV3ToolNames()
	found := false
	for _, name := range toolNames {
		if name == "test_tool" {
			found = true
			break
		}
	}
	
	if !found {
		t.Error("test_tool not found in V3 tool names")
	}
}

// TestSchemaGeneration verifies basic schema generation works
func TestSchemaGeneration(t *testing.T) {
	type SimpleStruct struct {
		Name     string `json:"name" jsonschema:"required,description=User name"`
		Age      int    `json:"age,omitempty" jsonschema:"minimum=0"`
		Optional string `json:"optional,omitempty"`
	}
	
	schema := generateSchema[SimpleStruct]()
	
	// Verify basic schema structure
	if schema["type"] != "object" {
		t.Errorf("Expected type 'object', got %v", schema["type"])
	}
	
	properties, ok := schema["properties"].(map[string]interface{})
	if !ok {
		t.Error("Schema properties should be a map")
		return
	}
	
	// Check that we have properties (even if stubbed)
	if len(properties) == 0 {
		t.Error("Schema should have properties")
	}
}

// TestResponseBuilder verifies the response builder works
func TestResponseBuilder(t *testing.T) {
	// Test basic text response
	response := NewResponse().AddText("Hello, World!").Build()
	
	if len(response.Content) != 1 {
		t.Errorf("Expected 1 content item, got %d", len(response.Content))
	}
	
	if response.Content[0].Type != ContentTypeText {
		t.Errorf("Expected text content type, got %s", response.Content[0].Type)
	}
	
	if response.Content[0].Text != "Hello, World!" {
		t.Errorf("Expected 'Hello, World!', got %s", response.Content[0].Text)
	}
	
	// Test error response
	errorResponse := NewResponse().AddText("Error occurred").WithError().Build()
	
	if !errorResponse.IsError {
		t.Error("Response should be marked as error")
	}
}

// TestErrorHandling verifies V3 error types work correctly
func TestErrorHandling(t *testing.T) {
	// Test ValidationError
	valErr := InvalidInput("field", "validation failed")
	
	if valErr.Error() == "" {
		t.Error("ValidationError should have error message")
	}
	
	// Test ToolError
	toolErr := ToolFailed("operation failed", valErr)
	
	if toolErr.Error() == "" {
		t.Error("ToolError should have error message")
	}
}

// TestV3APIInfo verifies API introspection works
func TestV3APIInfo(t *testing.T) {
	info := GetV3APIInfo()
	
	if version, ok := info["version"].(string); !ok || version != FTLSDKVersionV3 {
		t.Errorf("Expected version %s, got %v", FTLSDKVersionV3, info["version"])
	}
	
	if features, ok := info["features"].([]string); !ok || len(features) == 0 {
		t.Error("API info should include features list")
	}
}

// TestContextTypes verifies context types are working
func TestContextTypes(t *testing.T) {
	ctx := NewToolContext("test_tool")
	
	if ctx.ToolName != "test_tool" {
		t.Errorf("Expected tool name 'test_tool', got %s", ctx.ToolName)
	}
	
	if ctx.RequestID == "" {
		t.Error("RequestID should be generated")
	}
	
	if ctx.StartTime.IsZero() {
		t.Error("StartTime should be set")
	}
}

// TestCompilation verifies that all V3 types compile correctly
func TestCompilation(t *testing.T) {
	// This test mainly verifies that all our types compile
	// and basic functions can be called without panicking
	
	// Test all main type constructors
	_ = NewResponse()
	_ = NewToolContext("test")
	_ = GetV3APIInfo()
	
	// Test error constructors
	_ = InvalidInput("field", "message")
	_ = InternalError("message")
	
	// Test response helpers
	_ = TextResponse("text")
	_ = ErrorResponse("error")
	_ = StructuredResponse("", map[string]interface{}{"key": "value"})
	
	t.Log("All V3 types compile and basic functions work")
}

// TestHTTPIntegration_ToolRegistration tests HTTP interface integration for tool registration
func TestHTTPIntegration_ToolRegistration(t *testing.T) {
	type HTTPTestInput struct {
		URL    string            `json:"url" jsonschema:"required,description=URL to fetch"`
		Method string            `json:"method,omitempty" jsonschema:"enum=GET,POST,PUT,DELETE"`
		Headers map[string]string `json:"headers,omitempty"`
	}
	
	type HTTPTestOutput struct {
		StatusCode int               `json:"status_code"`
		Headers    map[string]string `json:"headers"`
		Body       string            `json:"body,omitempty"`
	}
	
	handler := func(ctx context.Context, input HTTPTestInput) (HTTPTestOutput, error) {
		if input.URL == "" {
			return HTTPTestOutput{}, InvalidInput("url", "URL is required")
		}
		
		// Simulate HTTP response
		return HTTPTestOutput{
			StatusCode: 200,
			Headers:    map[string]string{"Content-Type": "application/json"},
			Body:       `{"status": "ok"}`,
		}, nil
	}
	
	// Clear registry and register the HTTP test tool
	clearV3Registry()
	HandleTypedTool("http_test", handler)
	
	// Verify tool registration with HTTP-like complex types
	if !IsV3Tool("http_test") {
		t.Error("HTTP test tool should be registered")
	}
	
	// Get tool definition and verify schema generation for complex types
	tool, exists := v3Registry.GetTypedTool("http_test")
	if !exists {
		t.Fatal("HTTP test tool should exist in registry")
	}
	
	// Verify input schema handles complex types
	schema := tool.InputSchema
	if schema["type"] != "object" {
		t.Errorf("HTTP tool schema should be object type, got %v", schema["type"])
	}
	
	properties, ok := schema["properties"].(map[string]interface{})
	if !ok {
		t.Fatal("HTTP tool schema should have properties")
	}
	
	// Check URL field
	if urlField, ok := properties["url"]; !ok {
		t.Error("HTTP tool schema should have url field")
	} else if urlMap, ok := urlField.(map[string]interface{}); ok {
		if urlMap["type"] != "string" {
			t.Errorf("URL field should be string type, got %v", urlMap["type"])
		}
	}
	
	// Check headers field (should be object/map type)
	if headersField, ok := properties["headers"]; !ok {
		t.Error("HTTP tool schema should have headers field")
	} else if headersMap, ok := headersField.(map[string]interface{}); ok {
		if headersMap["type"] != "object" {
			t.Errorf("Headers field should be object type, got %v", headersMap["type"])
		}
	}
}

// TestHTTPIntegration_ComplexDataFlow tests complex data flow through HTTP interface
func TestHTTPIntegration_ComplexDataFlow(t *testing.T) {
	type Address struct {
		Street  string `json:"street" jsonschema:"required"`
		City    string `json:"city" jsonschema:"required"`
		Country string `json:"country" jsonschema:"required"`
		Zip     string `json:"zip" jsonschema:"pattern=^[0-9]{5}$"`
	}
	
	type Person struct {
		Name      string    `json:"name" jsonschema:"required,description=Full name"`
		Age       int       `json:"age" jsonschema:"minimum=0,maximum=150"`
		Email     string    `json:"email" jsonschema:"format=email"`
		Addresses []Address `json:"addresses,omitempty"`
		Metadata  map[string]interface{} `json:"metadata,omitempty"`
	}
	
	type DatabaseResult struct {
		PersonID    string `json:"person_id"`
		Created     string `json:"created"`
		UpdatedAt   string `json:"updated_at"`
		RecordCount int    `json:"record_count"`
	}
	
	handler := func(ctx context.Context, input Person) (DatabaseResult, error) {
		// Validate complex nested data
		if input.Name == "" {
			return DatabaseResult{}, InvalidInput("name", "name is required")
		}
		
		if input.Email != "" && !strings.Contains(input.Email, "@") {
			return DatabaseResult{}, InvalidInput("email", "email format is invalid")
		}
		
		// Validate nested addresses
		for i, addr := range input.Addresses {
			if addr.Street == "" {
				return DatabaseResult{}, InvalidInput("addresses", fmt.Sprintf("address %d is missing street", i))
			}
			if addr.City == "" {
				return DatabaseResult{}, InvalidInput("addresses", fmt.Sprintf("address %d is missing city", i))
			}
		}
		
		// Simulate database operation
		return DatabaseResult{
			PersonID:    "person_123",
			Created:     time.Now().Format(time.RFC3339),
			UpdatedAt:   time.Now().Format(time.RFC3339),
			RecordCount: len(input.Addresses),
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("person_db", handler)
	
	// Test complex data flow
	tool, exists := v3Registry.GetTypedTool("person_db")
	if !exists {
		t.Fatal("Person DB tool should exist")
	}
	
	// Test with valid complex input
	complexInput := map[string]interface{}{
		"name":  "John Doe",
		"age":   30,
		"email": "john@example.com",
		"addresses": []map[string]interface{}{
			{
				"street":  "123 Main St",
				"city":    "New York",
				"country": "USA",
				"zip":     "10001",
			},
			{
				"street":  "456 Oak Ave", 
				"city":    "Boston",
				"country": "USA",
				"zip":     "02101",
			},
		},
		"metadata": map[string]interface{}{
			"source":    "api",
			"validated": true,
			"tags":      []string{"customer", "premium"},
		},
	}
	
	response := tool.Handler(complexInput)
	
	// Should handle complex input without errors (in stub phase)
	if len(response.Content) == 0 {
		t.Error("Handler should return content for complex input")
	}
	
	// In RUN phase, verify structured response contains expected fields
	// TODO: Verify PersonID, Created, UpdatedAt, RecordCount in response
}

// TestHTTPIntegration_ErrorPropagation tests error propagation through HTTP interface
func TestHTTPIntegration_ErrorPropagation(t *testing.T) {
	type ErrorTestInput struct {
		Operation string `json:"operation" jsonschema:"required,enum=success,validation_error,internal_error,timeout"`
		Data      string `json:"data,omitempty"`
	}
	
	type ErrorTestOutput struct {
		Result string `json:"result"`
		Status string `json:"status"`
	}
	
	handler := func(ctx context.Context, input ErrorTestInput) (ErrorTestOutput, error) {
		switch input.Operation {
		case "success":
			return ErrorTestOutput{Result: "operation successful", Status: "ok"}, nil
		case "validation_error":
			return ErrorTestOutput{}, InvalidInput("data", "data validation failed")
		case "internal_error":
			return ErrorTestOutput{}, InternalError("internal processing error")
		case "timeout":
			return ErrorTestOutput{}, NewToolError("TIMEOUT", "operation timed out")
		default:
			return ErrorTestOutput{}, InvalidInput("operation", "unknown operation")
		}
	}
	
	clearV3Registry()
	HandleTypedTool("error_test", handler)
	
	tool, exists := v3Registry.GetTypedTool("error_test")
	if !exists {
		t.Fatal("Error test tool should exist")
	}
	
	// Test successful operation
	successInput := map[string]interface{}{"operation": "success"}
	response := tool.Handler(successInput)
	
	if response.IsError {
		t.Error("Success operation should not return error")
	}
	
	// Test validation error
	validationInput := map[string]interface{}{"operation": "validation_error"}
	response = tool.Handler(validationInput)
	
	// Should handle validation error gracefully (specific behavior depends on RUN phase implementation)
	if len(response.Content) == 0 {
		t.Error("Validation error should return some content")
	}
	
	// Test internal error
	internalInput := map[string]interface{}{"operation": "internal_error"}
	response = tool.Handler(internalInput)
	
	// Should handle internal error gracefully
	if len(response.Content) == 0 {
		t.Error("Internal error should return some content")
	}
	
	// Test timeout error
	timeoutInput := map[string]interface{}{"operation": "timeout"}
	response = tool.Handler(timeoutInput)
	
	// Should handle timeout error gracefully
	if len(response.Content) == 0 {
		t.Error("Timeout error should return some content")
	}
}

// TestHTTPIntegration_ConcurrentRequests tests concurrent HTTP request handling
func TestHTTPIntegration_ConcurrentRequests(t *testing.T) {
	type ConcurrentTestInput struct {
		WorkerID int    `json:"worker_id" jsonschema:"required,minimum=1"`
		Task     string `json:"task" jsonschema:"required"`
		Duration int    `json:"duration_ms,omitempty" jsonschema:"minimum=0,maximum=1000"`
	}
	
	type ConcurrentTestOutput struct {
		WorkerID    int    `json:"worker_id"`
		Result      string `json:"result"`
		ProcessedAt string `json:"processed_at"`
	}
	
	handler := func(ctx context.Context, input ConcurrentTestInput) (ConcurrentTestOutput, error) {
		// Simulate work duration
		if input.Duration > 0 {
			time.Sleep(time.Duration(input.Duration) * time.Millisecond)
		}
		
		return ConcurrentTestOutput{
			WorkerID:    input.WorkerID,
			Result:      fmt.Sprintf("Worker %d completed task: %s", input.WorkerID, input.Task),
			ProcessedAt: time.Now().Format(time.RFC3339),
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("concurrent_test", handler)
	
	tool, exists := v3Registry.GetTypedTool("concurrent_test")
	if !exists {
		t.Fatal("Concurrent test tool should exist")
	}
	
	// Test concurrent execution
	const numWorkers = 10
	results := make(chan ToolResponse, numWorkers)
	
	for i := 1; i <= numWorkers; i++ {
		go func(workerID int) {
			input := map[string]interface{}{
				"worker_id": workerID,
				"task":      fmt.Sprintf("task_%d", workerID),
				"duration":  50, // 50ms work simulation
			}
			
			response := tool.Handler(input)
			results <- response
		}(i)
	}
	
	// Collect all results
	var responses []ToolResponse
	for i := 0; i < numWorkers; i++ {
		response := <-results
		responses = append(responses, response)
	}
	
	// Verify all workers completed
	if len(responses) != numWorkers {
		t.Errorf("Expected %d responses, got %d", numWorkers, len(responses))
	}
	
	// Verify no errors in concurrent execution
	errorCount := 0
	for _, response := range responses {
		if response.IsError {
			errorCount++
		}
	}
	
	if errorCount > 0 {
		t.Errorf("Expected 0 errors in concurrent execution, got %d", errorCount)
	}
}

// TestHTTPIntegration_LargePayloads tests handling of large HTTP payloads
func TestHTTPIntegration_LargePayloads(t *testing.T) {
	type LargePayloadInput struct {
		Data        []byte            `json:"data" jsonschema:"description=Large binary data"`
		Metadata    map[string]string `json:"metadata,omitempty"`
		ChunkSize   int               `json:"chunk_size,omitempty" jsonschema:"minimum=1024,maximum=65536"`
	}
	
	type LargePayloadOutput struct {
		DataSize    int    `json:"data_size"`
		Checksum    string `json:"checksum"`
		ProcessedAt string `json:"processed_at"`
	}
	
	handler := func(ctx context.Context, input LargePayloadInput) (LargePayloadOutput, error) {
		if len(input.Data) == 0 {
			return LargePayloadOutput{}, InvalidInput("data", "data cannot be empty")
		}
		
		// Simulate checksum calculation
		checksum := fmt.Sprintf("sha256_%d", len(input.Data))
		
		return LargePayloadOutput{
			DataSize:    len(input.Data),
			Checksum:    checksum,
			ProcessedAt: time.Now().Format(time.RFC3339),
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("large_payload", handler)
	
	tool, exists := v3Registry.GetTypedTool("large_payload")
	if !exists {
		t.Fatal("Large payload tool should exist")
	}
	
	// Test with large data (1MB)
	largeData := make([]byte, 1024*1024)
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}
	
	input := map[string]interface{}{
		"data": largeData,
		"metadata": map[string]string{
			"source":      "test",
			"compression": "none",
		},
		"chunk_size": 8192,
	}
	
	response := tool.Handler(input)
	
	// Should handle large payload without panicking
	if len(response.Content) == 0 {
		t.Error("Large payload handler should return content")
	}
	
	// TODO: In RUN phase, verify DataSize equals len(largeData)
	// TODO: In RUN phase, verify Checksum is calculated correctly
}

// TestHTTPIntegration_ContentTypes tests different content types in responses
func TestHTTPIntegration_ContentTypes(t *testing.T) {
	type ContentTypeInput struct {
		ContentType string `json:"content_type" jsonschema:"required,enum=text,image,audio,resource,structured"`
		Content     string `json:"content" jsonschema:"required"`
	}
	
	type ContentTypeOutput struct {
		GeneratedContent string `json:"generated_content"`
		ContentType      string `json:"content_type"`
	}
	
	handler := func(ctx context.Context, input ContentTypeInput) (ContentTypeOutput, error) {
		// This handler will use different response building patterns
		// based on the requested content type
		return ContentTypeOutput{
			GeneratedContent: fmt.Sprintf("Generated %s content: %s", input.ContentType, input.Content),
			ContentType:      input.ContentType,
		}, nil
	}
	
	clearV3Registry()
	HandleTypedTool("content_type", handler)
	
	tool, exists := v3Registry.GetTypedTool("content_type")
	if !exists {
		t.Fatal("Content type tool should exist")
	}
	
	// Test different content type requests
	contentTypes := []string{"text", "image", "audio", "resource", "structured"}
	
	for _, contentType := range contentTypes {
		input := map[string]interface{}{
			"content_type": contentType,
			"content":      fmt.Sprintf("test %s content", contentType),
		}
		
		response := tool.Handler(input)
		
		// Should handle different content type requests
		if len(response.Content) == 0 {
			t.Errorf("Content type %s handler should return content", contentType)
		}
		
		// TODO: In RUN phase, verify response contains appropriate content type
		// TODO: Test that ResponseBuilder methods work with different content types
	}
}