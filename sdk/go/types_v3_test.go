package ftl

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"
)

// TestToolContext tests the ToolContext structure and methods
func TestToolContext(t *testing.T) {
	ctx := context.Background()
	requestID := "test-request-123"
	startTime := time.Now()
	
	toolCtx := &ToolContext{
		Context:   ctx,
		ToolName:  "test_tool",
		RequestID: requestID,
		StartTime: startTime,
	}
	
	// Test basic fields
	if toolCtx.Context != ctx {
		t.Error("ToolContext should preserve original context")
	}
	
	if toolCtx.RequestID != requestID {
		t.Errorf("Expected RequestID '%s', got '%s'", requestID, toolCtx.RequestID)
	}
	
	if !toolCtx.StartTime.Equal(startTime) {
		t.Errorf("Expected start time %v, got %v", startTime, toolCtx.StartTime)
	}
	
	// ToolContext doesn't have a Meta field in the actual implementation
	// Test that we can use context.Value for metadata instead
	type ctxKey string
	const userKey ctxKey = "user"
	const sessionKey ctxKey = "session"
	
	ctxWithMeta := context.WithValue(toolCtx.Context, userKey, "test-user")
	ctxWithMeta = context.WithValue(ctxWithMeta, sessionKey, "session-456")
	toolCtx.Context = ctxWithMeta
	
	if toolCtx.Context.Value(userKey) != "test-user" {
		t.Error("Context should store user value")
	}
	
	if toolCtx.Context.Value(sessionKey) != "session-456" {
		t.Error("Context should store session value")
	}
}

// TestToolContext_WithTimeout tests context timeout functionality
func TestToolContext_WithTimeout(t *testing.T) {
	baseCtx := context.Background()
	toolCtx := &ToolContext{
		Context:   baseCtx,
		ToolName:  "timeout_test",
		RequestID: "timeout-test",
		StartTime: time.Now(),
	}
	
	// Create context with timeout
	timeoutCtx, cancel := context.WithTimeout(toolCtx.Context, 100*time.Millisecond)
	defer cancel()
	
	toolCtx.Context = timeoutCtx
	
	// Test that context timeout is preserved
	select {
	case <-toolCtx.Context.Done():
		// Expected after timeout
	case <-time.After(200 * time.Millisecond):
		t.Error("Context should have timed out")
	}
	
	// Check timeout error
	err := toolCtx.Context.Err()
	if err != context.DeadlineExceeded {
		t.Errorf("Expected context.DeadlineExceeded, got %v", err)
	}
}

// TestToolContext_WithCancel tests context cancellation
func TestToolContext_WithCancel(t *testing.T) {
	baseCtx := context.Background()
	cancelCtx, cancel := context.WithCancel(baseCtx)
	
	toolCtx := &ToolContext{
		Context:   cancelCtx,
		ToolName:  "cancel_test",
		RequestID: "cancel-test",
		StartTime: time.Now(),
	}
	
	// Context should not be cancelled initially
	select {
	case <-toolCtx.Context.Done():
		t.Error("Context should not be cancelled initially")
	default:
		// Expected
	}
	
	// Cancel the context
	cancel()
	
	// Context should now be cancelled
	select {
	case <-toolCtx.Context.Done():
		// Expected
	case <-time.After(100 * time.Millisecond):
		t.Error("Context should be cancelled")
	}
	
	// Check cancellation error
	err := toolCtx.Context.Err()
	if err != context.Canceled {
		t.Errorf("Expected context.Canceled, got %v", err)
	}
}

// TestValidationError tests ValidationError creation and behavior
func TestValidationError(t *testing.T) {
	field := "username"
	message := "username must be at least 3 characters"
	
	err := ValidationError{Field: field, Message: message}
	
	if err.Field != field {
		t.Errorf("Expected field '%s', got '%s'", field, err.Field)
	}
	
	if err.Message != message {
		t.Errorf("Expected message '%s', got '%s'", message, err.Message)
	}
	
	// Test Error() method
	expectedError := "validation failed for field 'username': username must be at least 3 characters"
	if err.Error() != expectedError {
		t.Errorf("Expected error string '%s', got '%s'", expectedError, err.Error())
	}
}

// TestValidationError_MultipleFields tests validation errors for multiple fields
func TestValidationError_MultipleFields(t *testing.T) {
	errors := []ValidationError{
		ValidationError{Field: "email", Message: "email is required"},
		ValidationError{Field: "password", Message: "password must be at least 8 characters"},
		ValidationError{Field: "age", Message: "age must be between 18 and 100"},
	}
	
	// Test that each error has correct field and message
	expectedFields := []string{"email", "password", "age"}
	expectedMessages := []string{
		"email is required",
		"password must be at least 8 characters",
		"age must be between 18 and 100",
	}
	
	for i, err := range errors {
		if err.Field != expectedFields[i] {
			t.Errorf("Error %d: expected field '%s', got '%s'", i, expectedFields[i], err.Field)
		}
		
		if err.Message != expectedMessages[i] {
			t.Errorf("Error %d: expected message '%s', got '%s'", i, expectedMessages[i], err.Message)
		}
	}
}

// TestToolError tests ToolError creation and behavior
func TestToolError(t *testing.T) {
	code := "PROCESSING_ERROR"
	message := "Failed to process request"
	
	err := NewToolError(code, message)
	
	// NewToolError returns error interface, need type assertion
	toolErr, ok := err.(ToolError)
	if !ok {
		t.Fatal("NewToolError should return ToolError type")
	}
	
	if toolErr.Code != code {
		t.Errorf("Expected code '%s', got '%s'", code, toolErr.Code)
	}
	
	if toolErr.Message != message {
		t.Errorf("Expected message '%s', got '%s'", message, toolErr.Message)
	}
	
	// Test Error() method
	if !strings.Contains(err.Error(), message) {
		t.Errorf("Error string should contain message '%s', got '%s'", message, err.Error())
	}
}

// TestToolError_CommonErrorCodes tests common error code patterns
func TestToolError_CommonErrorCodes(t *testing.T) {
	commonErrors := []error{
		NewToolError("INVALID_INPUT", "Input validation failed"),
		NewToolError("RESOURCE_NOT_FOUND", "Requested resource does not exist"),
		NewToolError("PERMISSION_DENIED", "Insufficient permissions"),
		NewToolError("RATE_LIMIT_EXCEEDED", "Too many requests"),
		NewToolError("INTERNAL_ERROR", "Internal processing error"),
		NewToolError("TIMEOUT", "Operation timed out"),
		NewToolError("NETWORK_ERROR", "Network communication failed"),
	}
	
	expectedCodes := []string{
		"INVALID_INPUT",
		"RESOURCE_NOT_FOUND", 
		"PERMISSION_DENIED",
		"RATE_LIMIT_EXCEEDED",
		"INTERNAL_ERROR",
		"TIMEOUT",
		"NETWORK_ERROR",
	}
	
	for i, err := range commonErrors {
		toolErr, ok := err.(ToolError)
		if !ok {
			t.Errorf("Error %d: should be ToolError type", i)
			continue
		}
		
		if toolErr.Code != expectedCodes[i] {
			t.Errorf("Error %d: expected code '%s', got '%s'", i, expectedCodes[i], toolErr.Code)
		}
		
		if toolErr.Message == "" {
			t.Errorf("Error %d: message should not be empty", i)
		}
	}
}

// TestV3ToolRegistry tests the V3 tool registry
func TestV3ToolRegistry(t *testing.T) {
	registry := &V3ToolRegistry{
		tools: make(map[string]TypedToolDefinition),
	}
	
	// Test empty registry
	if len(registry.tools) != 0 {
		t.Error("New registry should be empty")
	}
	
	// Test adding a tool
	toolDef := TypedToolDefinition{
		ToolDefinition: ToolDefinition{
			Name:        "mock_tool",
			InputSchema: map[string]interface{}{"type": "object"},
			Meta:        map[string]interface{}{"ftl_sdk_version": "v3"},
		},
		InputType:  "MockInput",
		OutputType: "MockOutput",
	}
	
	registry.RegisterTypedTool("mock_tool", toolDef)
	
	// Test registry now contains tool
	if len(registry.tools) != 1 {
		t.Errorf("Registry should contain 1 tool, got %d", len(registry.tools))
	}
	
	// Test getting typed tool
	retrievedTool, exists := registry.GetTypedTool("mock_tool")
	if !exists {
		t.Error("Tool should exist in registry")
	}
	
	if retrievedTool.Name != "mock_tool" {
		t.Errorf("Retrieved tool name should be 'mock_tool', got '%s'", retrievedTool.Name)
	}
	
	// Test getting non-existent tool
	_, exists = registry.GetTypedTool("non_existent")
	if exists {
		t.Error("Non-existent tool should not exist in registry")
	}
}

// TestV3ToolRegistry_Concurrent tests concurrent access to the registry
func TestV3ToolRegistry_Concurrent(t *testing.T) {
	registry := &V3ToolRegistry{
		tools: make(map[string]TypedToolDefinition),
	}
	
	// Create multiple goroutines that add tools concurrently
	done := make(chan bool, 10)
	
	for i := 0; i < 10; i++ {
		go func(id int) {
			toolName := fmt.Sprintf("tool_%d", id)
			toolDef := TypedToolDefinition{
				ToolDefinition: ToolDefinition{
					Name:        toolName,
					InputSchema: map[string]interface{}{"type": "object"},
					Meta:        map[string]interface{}{"id": id},
				},
				InputType:  "Input",
				OutputType: "Output",
			}
			
			registry.RegisterTypedTool(toolName, toolDef)
			done <- true
		}(i)
	}
	
	// Wait for all goroutines to complete
	for i := 0; i < 10; i++ {
		<-done
	}
	
	// Verify all tools were added
	if len(registry.tools) != 10 {
		t.Errorf("Expected 10 tools in registry, got %d", len(registry.tools))
	}
}

// TestTypedToolDefinition tests the TypedToolDefinition structure
func TestTypedToolDefinition(t *testing.T) {
	schema := map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"message": map[string]interface{}{
				"type": "string",
			},
		},
		"required": []string{"message"},
	}
	
	meta := map[string]interface{}{
		"ftl_sdk_version": "v3",
		"type_safe":       true,
		"created_at":      time.Now().Format(time.RFC3339),
	}
	
	toolDef := TypedToolDefinition{
		ToolDefinition: ToolDefinition{
			Name:        "test_tool",
			InputSchema: schema,
			Meta:        meta,
		},
		InputType:  "TestInput",
		OutputType: "TestOutput",
	}
	
	// Test basic fields
	if toolDef.Name != "test_tool" {
		t.Errorf("Expected name 'test_tool', got '%s'", toolDef.Name)
	}
	
	if toolDef.InputSchema == nil {
		t.Error("InputSchema should not be nil")
	}
	
	if toolDef.Meta == nil {
		t.Error("Meta should not be nil")
	}
	
	// Test schema structure
	if toolDef.InputSchema["type"] != "object" {
		t.Errorf("Schema type should be 'object', got %v", toolDef.InputSchema["type"])
	}
	
	// Test meta data
	if toolDef.Meta["ftl_sdk_version"] != "v3" {
		t.Errorf("Meta ftl_sdk_version should be 'v3', got %v", toolDef.Meta["ftl_sdk_version"])
	}
	
	if toolDef.Meta["type_safe"] != true {
		t.Errorf("Meta type_safe should be true, got %v", toolDef.Meta["type_safe"])
	}
	
	// Test type fields
	if toolDef.InputType != "TestInput" {
		t.Errorf("Expected InputType 'TestInput', got '%s'", toolDef.InputType)
	}
	
	if toolDef.OutputType != "TestOutput" {
		t.Errorf("Expected OutputType 'TestOutput', got '%s'", toolDef.OutputType)
	}
}

// TestInvalidInput tests the InvalidInput helper function
func TestInvalidInput(t *testing.T) {
	field := "email"
	message := "email format is invalid"
	
	err := InvalidInput(field, message)
	
	// Should return a ValidationError
	validationErr, ok := err.(ValidationError)
	if !ok {
		t.Fatal("InvalidInput should return ValidationError")
	}
	
	if validationErr.Field != field {
		t.Errorf("Expected field '%s', got '%s'", field, validationErr.Field)
	}
	
	if validationErr.Message != message {
		t.Errorf("Expected message '%s', got '%s'", message, validationErr.Message)
	}
}

// TestInternalError tests the InternalError helper function
func TestInternalError(t *testing.T) {
	message := "database connection failed"
	
	err := InternalError(message)
	
	// Should return a ToolError with internal_error code
	toolErr, ok := err.(ToolError)
	if !ok {
		t.Fatal("InternalError should return ToolError")
	}
	
	if toolErr.Code != "internal_error" {
		t.Errorf("Expected code 'internal_error', got '%s'", toolErr.Code)
	}
	
	if toolErr.Message != message {
		t.Errorf("Expected message '%s', got '%s'", message, toolErr.Message)
	}
}

// TestErrorHelpers tests various error helper functions
func TestErrorHelpers(t *testing.T) {
	// Test InvalidInput
	invalidErr := InvalidInput("name", "name is required")
	if invalidErr.Error() == "" {
		t.Error("InvalidInput should return non-empty error string")
	}
	
	// Test InternalError
	internalErr := InternalError("database error")
	toolErr, ok := internalErr.(ToolError)
	if !ok {
		t.Fatal("InternalError should return ToolError")
	}
	if toolErr.Code != "internal_error" {
		t.Error("InternalError should have internal_error code")
	}
	
	// Test NewToolError with custom code
	customErr := NewToolError("CUSTOM_ERROR", "custom error message")
	toolErr, ok = customErr.(ToolError)
	if !ok {
		t.Fatal("NewToolError should return ToolError")
	}
	if toolErr.Code != "CUSTOM_ERROR" {
		t.Error("NewToolError should preserve custom code")
	}
	
	// Test ToolFailed
	cause := fmt.Errorf("underlying error")
	failedErr := ToolFailed("operation failed", cause)
	toolErr, ok = failedErr.(ToolError)
	if !ok {
		t.Fatal("ToolFailed should return ToolError")
	}
	if toolErr.Code != "execution_failed" {
		t.Error("ToolFailed should have execution_failed code")
	}
	if toolErr.Cause != cause {
		t.Error("ToolFailed should preserve cause")
	}
}

// TestGlobalRegistryFunctions tests global V3 registry functions
func TestGlobalRegistryFunctions(t *testing.T) {
	// Clear registry first by reassigning
	registeredV3ToolsMu.Lock()
	registeredV3Tools = make(map[string]bool)
	registeredV3ToolsMu.Unlock()
	
	// Test with empty registry
	registeredV3ToolsMu.RLock()
	exists := registeredV3Tools["non_existent"]
	registeredV3ToolsMu.RUnlock()
	if exists {
		t.Error("Registry should not contain non-existent tool")
	}
	
	// Add a tool to registry (simulate via global map)
	registeredV3ToolsMu.Lock()
	registeredV3Tools["test_tool"] = true
	registeredV3ToolsMu.Unlock()
	
	// Test with existing tool
	registeredV3ToolsMu.RLock()
	exists = registeredV3Tools["test_tool"]
	registeredV3ToolsMu.RUnlock()
	if !exists {
		t.Error("Registry should contain registered tool")
	}
	
	// Count tools in registry
	count := 0
	registeredV3ToolsMu.RLock()
	for range registeredV3Tools {
		count++
	}
	registeredV3ToolsMu.RUnlock()
	
	if count != 1 {
		t.Errorf("Registry should have 1 tool, got %d", count)
	}
}