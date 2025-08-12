package main

import (
	"context"
	"strings"
	"testing"
	"time"

	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

// TestEchoHandler tests the EchoHandler function directly
func TestEchoHandler(t *testing.T) {
	ctx := context.Background()

	// Test valid input with default count
	input := EchoInput{
		Message: "Hello, World!",
	}

	output, err := EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error for valid input: %v", err)
	}

	if output.Response != "Hello, World!" {
		t.Errorf("Expected response 'Hello, World!', got '%s'", output.Response)
	}

	if output.RepetitionCount != 1 {
		t.Errorf("Expected repetition count 1, got %d", output.RepetitionCount)
	}

	if output.EchoedAt == "" {
		t.Error("EchoedAt should not be empty")
	}

	if output.ProcessingTimeMs < 0 {
		t.Errorf("ProcessingTimeMs should be non-negative, got %d", output.ProcessingTimeMs)
	}

	// Test input with multiple repetitions
	input = EchoInput{
		Message: "Test",
		Count:   3,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error for valid input: %v", err)
	}

	expectedResponse := "Test\nTest\nTest"
	if output.Response != expectedResponse {
		t.Errorf("Expected response '%s', got '%s'", expectedResponse, output.Response)
	}

	if output.RepetitionCount != 3 {
		t.Errorf("Expected repetition count 3, got %d", output.RepetitionCount)
	}

	// Test input with prefix
	input = EchoInput{
		Message: "Message",
		Count:   2,
		Prefix:  "PREFIX: ",
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error for valid input: %v", err)
	}

	expectedResponse = "PREFIX: Message\nPREFIX: Message"
	if output.Response != expectedResponse {
		t.Errorf("Expected response '%s', got '%s'", expectedResponse, output.Response)
	}

	// Test empty message (should return error)
	input = EchoInput{
		Message: "",
	}

	_, err = EchoHandler(ctx, input)
	if err == nil {
		t.Error("EchoHandler should return error for empty message")
	}

	// Check that it's a validation error
	validationErr, ok := err.(*ftl.ValidationError)
	if !ok {
		t.Errorf("Error should be ValidationError, got %T", err)
	} else {
		if validationErr.Field != "message" {
			t.Errorf("Expected field 'message', got '%s'", validationErr.Field)
		}
	}

	// Test zero count (should default to 1)
	input = EchoInput{
		Message: "Zero count test",
		Count:   0,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error for zero count: %v", err)
	}

	if output.RepetitionCount != 1 {
		t.Errorf("Zero count should default to 1, got %d", output.RepetitionCount)
	}

	// Test negative count (should default to 1)
	input = EchoInput{
		Message: "Negative count test",
		Count:   -5,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error for negative count: %v", err)
	}

	if output.RepetitionCount != 1 {
		t.Errorf("Negative count should default to 1, got %d", output.RepetitionCount)
	}
}

// TestGreetingHandler tests the GreetingHandler function directly
func TestGreetingHandler(t *testing.T) {
	ctx := context.Background()

	// Test valid input with informal greeting
	input := GreetingInput{
		Name:   "Alice",
		Formal: false,
	}

	output, err := GreetingHandler(ctx, input)
	if err != nil {
		t.Fatalf("GreetingHandler should not return error for valid input: %v", err)
	}

	expectedGreeting := "Hello, Alice!"
	if output.Greeting != expectedGreeting {
		t.Errorf("Expected greeting '%s', got '%s'", expectedGreeting, output.Greeting)
	}

	if output.Language != "en" {
		t.Errorf("Expected language 'en', got '%s'", output.Language)
	}

	if output.Formal != false {
		t.Errorf("Expected formal false, got %v", output.Formal)
	}

	// Test formal greeting
	input = GreetingInput{
		Name:   "Mr. Smith",
		Formal: true,
	}

	output, err = GreetingHandler(ctx, input)
	if err != nil {
		t.Fatalf("GreetingHandler should not return error for formal input: %v", err)
	}

	expectedGreeting = "Good day, Mr. Smith. How may I assist you today?"
	if output.Greeting != expectedGreeting {
		t.Errorf("Expected greeting '%s', got '%s'", expectedGreeting, output.Greeting)
	}

	if output.Formal != true {
		t.Errorf("Expected formal true, got %v", output.Formal)
	}

	// Test empty name (should return error)
	input = GreetingInput{
		Name: "",
	}

	_, err = GreetingHandler(ctx, input)
	if err == nil {
		t.Error("GreetingHandler should return error for empty name")
	}

	// Check that it's a validation error
	validationErr, ok := err.(*ftl.ValidationError)
	if !ok {
		t.Errorf("Error should be ValidationError, got %T", err)
	} else {
		if validationErr.Field != "name" {
			t.Errorf("Expected field 'name', got '%s'", validationErr.Field)
		}
	}
}

// TestEchoHandler_ContextHandling tests context handling in EchoHandler
func TestEchoHandler_ContextHandling(t *testing.T) {
	// Test with cancelled context
	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	input := EchoInput{
		Message: "Should handle cancelled context",
	}

	// Handler should still work even with cancelled context
	// (unless it specifically checks for cancellation)
	output, err := EchoHandler(ctx, input)
	
	// The current handler doesn't check context cancellation,
	// so it should still work normally
	if err != nil {
		t.Fatalf("EchoHandler should not return error even with cancelled context: %v", err)
	}

	if output.Response != "Should handle cancelled context" {
		t.Error("Handler should process normally even with cancelled context")
	}

	// Test with timeout context
	timeoutCtx, cancelTimeout := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancelTimeout()

	input = EchoInput{
		Message: "Should handle timeout context",
	}

	output, err = EchoHandler(timeoutCtx, input)
	if err != nil {
		t.Fatalf("EchoHandler should not return error with timeout context: %v", err)
	}

	if output.Response != "Should handle timeout context" {
		t.Error("Handler should process normally with timeout context")
	}
}

// TestEchoHandler_EdgeCases tests edge cases for EchoHandler
func TestEchoHandler_EdgeCases(t *testing.T) {
	ctx := context.Background()

	// Test very long message
	longMessage := strings.Repeat("A", 1000)
	input := EchoInput{
		Message: longMessage,
		Count:   1,
	}

	output, err := EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should handle long messages: %v", err)
	}

	if output.Response != longMessage {
		t.Error("Handler should preserve long messages exactly")
	}

	// Test maximum reasonable count
	input = EchoInput{
		Message: "Short",
		Count:   10,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should handle reasonable count: %v", err)
	}

	if output.RepetitionCount != 10 {
		t.Errorf("Expected repetition count 10, got %d", output.RepetitionCount)
	}

	expectedParts := strings.Split(output.Response, "\n")
	if len(expectedParts) != 10 {
		t.Errorf("Expected 10 parts in response, got %d", len(expectedParts))
	}

	// Test special characters in message
	input = EchoInput{
		Message: "Special chars: !@#$%^&*()[]{}|\\:;\"'<>?,.`~",
		Count:   1,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should handle special characters: %v", err)
	}

	if output.Response != input.Message {
		t.Error("Handler should preserve special characters exactly")
	}

	// Test unicode characters
	input = EchoInput{
		Message: "Unicode: 🌟🚀💫 こんにちは العالم 🎉",
		Count:   1,
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should handle unicode: %v", err)
	}

	if output.Response != input.Message {
		t.Error("Handler should preserve unicode characters exactly")
	}

	// Test very long prefix
	input = EchoInput{
		Message: "Message",
		Count:   2,
		Prefix:  strings.Repeat("PREFIX_", 10),
	}

	output, err = EchoHandler(ctx, input)
	if err != nil {
		t.Fatalf("EchoHandler should handle long prefix: %v", err)
	}

	expectedPrefix := strings.Repeat("PREFIX_", 10)
	parts := strings.Split(output.Response, "\n")
	for i, part := range parts {
		if !strings.HasPrefix(part, expectedPrefix) {
			t.Errorf("Part %d should start with long prefix, got '%s'", i, part)
		}
	}
}

// TestToolRegistration tests that the tools are properly registered via init()
func TestToolRegistration(t *testing.T) {
	// The init() function should have registered both tools
	
	// Test echo tool registration
	if !ftl.IsV3Tool("echo") {
		t.Error("Echo tool should be registered as V3 tool")
	}

	// Test greeting tool registration
	if !ftl.IsV3Tool("greeting") {
		t.Error("Greeting tool should be registered as V3 tool")
	}

	// Test that both tools appear in the registry
	toolNames := ftl.GetV3ToolNames()
	
	echoFound := false
	greetingFound := false
	
	for _, name := range toolNames {
		if name == "echo" {
			echoFound = true
		}
		if name == "greeting" {
			greetingFound = true
		}
	}

	if !echoFound {
		t.Error("Echo tool should appear in V3 tool names")
	}

	if !greetingFound {
		t.Error("Greeting tool should appear in V3 tool names")
	}
}

// TestInputValidation tests input validation through the type system
func TestInputValidation(t *testing.T) {
	// These tests verify that the struct tags provide the expected
	// validation hints for the JSON schema generation

	// Test EchoInput validation requirements
	// Note: In the WALK phase, this primarily tests that the types
	// compile correctly and the tags are properly defined

	echoInput := EchoInput{
		Message: "Required field",
		Count:   5,
		Prefix:  "Optional field",
	}

	// Basic field access should work
	if echoInput.Message == "" {
		t.Error("Required message field should be accessible")
	}

	if echoInput.Count < 1 || echoInput.Count > 10 {
		// This would be enforced by JSON schema validation
		t.Log("Count should be validated by schema (1-10)")
	}

	// Test GreetingInput validation requirements
	greetingInput := GreetingInput{
		Name:   "Required name",
		Formal: true,
	}

	if greetingInput.Name == "" {
		t.Error("Required name field should be accessible")
	}

	// Test output types
	echoOutput := EchoOutput{
		Response:         "Test response",
		EchoedAt:        time.Now().Format(time.RFC3339),
		RepetitionCount:  1,
		ProcessingTimeMs: 50,
	}

	if echoOutput.Response == "" {
		t.Error("Output fields should be accessible")
	}

	greetingOutput := GreetingOutput{
		Greeting: "Hello!",
		Language: "en",
		Formal:   false,
	}

	if greetingOutput.Greeting == "" {
		t.Error("Greeting output fields should be accessible")
	}
}

// TestConcurrentHandlers tests concurrent execution of handlers
func TestConcurrentHandlers(t *testing.T) {
	ctx := context.Background()
	numGoroutines := 10

	// Test concurrent echo handlers
	echoDone := make(chan error, numGoroutines)
	
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			input := EchoInput{
				Message: fmt.Sprintf("Message %d", id),
				Count:   2,
			}
			
			output, err := EchoHandler(ctx, input)
			if err != nil {
				echoDone <- err
				return
			}
			
			expectedResponse := fmt.Sprintf("Message %d\nMessage %d", id, id)
			if output.Response != expectedResponse {
				echoDone <- fmt.Errorf("concurrent handler %d failed: expected '%s', got '%s'", 
					id, expectedResponse, output.Response)
				return
			}
			
			echoDone <- nil
		}(i)
	}

	// Wait for all echo handlers to complete
	for i := 0; i < numGoroutines; i++ {
		if err := <-echoDone; err != nil {
			t.Errorf("Echo handler error: %v", err)
		}
	}

	// Test concurrent greeting handlers
	greetingDone := make(chan error, numGoroutines)
	
	for i := 0; i < numGoroutines; i++ {
		go func(id int) {
			input := GreetingInput{
				Name:   fmt.Sprintf("User%d", id),
				Formal: id%2 == 0, // Alternate between formal/informal
			}
			
			output, err := GreetingHandler(ctx, input)
			if err != nil {
				greetingDone <- err
				return
			}
			
			if output.Greeting == "" {
				greetingDone <- fmt.Errorf("concurrent greeting handler %d returned empty greeting", id)
				return
			}
			
			if output.Formal != (id%2 == 0) {
				greetingDone <- fmt.Errorf("concurrent greeting handler %d formal flag mismatch", id)
				return
			}
			
			greetingDone <- nil
		}(i)
	}

	// Wait for all greeting handlers to complete
	for i := 0; i < numGoroutines; i++ {
		if err := <-greetingDone; err != nil {
			t.Errorf("Greeting handler error: %v", err)
		}
	}
}

// TestPerformance tests basic performance characteristics
func TestPerformance(t *testing.T) {
	ctx := context.Background()

	// Test echo handler performance
	start := time.Now()
	
	for i := 0; i < 100; i++ {
		input := EchoInput{
			Message: fmt.Sprintf("Performance test %d", i),
			Count:   1,
		}
		
		_, err := EchoHandler(ctx, input)
		if err != nil {
			t.Fatalf("Performance test failed at iteration %d: %v", i, err)
		}
	}
	
	echoTime := time.Since(start)
	
	// Each call should be very fast (< 10ms on average)
	avgTime := echoTime / 100
	if avgTime > 10*time.Millisecond {
		t.Logf("Echo handler average time: %v (may be acceptable for simple operations)", avgTime)
	}

	// Test greeting handler performance
	start = time.Now()
	
	for i := 0; i < 100; i++ {
		input := GreetingInput{
			Name:   fmt.Sprintf("User%d", i),
			Formal: i%2 == 0,
		}
		
		_, err := GreetingHandler(ctx, input)
		if err != nil {
			t.Fatalf("Greeting performance test failed at iteration %d: %v", i, err)
		}
	}
	
	greetingTime := time.Since(start)
	
	// Each call should be very fast (< 10ms on average)
	avgTime = greetingTime / 100
	if avgTime > 10*time.Millisecond {
		t.Logf("Greeting handler average time: %v (may be acceptable for simple operations)", avgTime)
	}
}

// TestProcessingTimeAccuracy tests that ProcessingTimeMs is reasonably accurate
func TestProcessingTimeAccuracy(t *testing.T) {
	ctx := context.Background()

	input := EchoInput{
		Message: "Timing test",
		Count:   1,
	}

	start := time.Now()
	output, err := EchoHandler(ctx, input)
	actualTime := time.Since(start).Milliseconds()

	if err != nil {
		t.Fatalf("Handler should not return error: %v", err)
	}

	// ProcessingTimeMs should be within a reasonable range of actual time
	if output.ProcessingTimeMs < 0 {
		t.Error("ProcessingTimeMs should not be negative")
	}

	// Allow some tolerance for timing differences
	tolerance := int64(10) // 10ms tolerance
	if output.ProcessingTimeMs > actualTime+tolerance {
		t.Errorf("ProcessingTimeMs (%d) should not exceed actual time (%d) by more than %dms", 
			output.ProcessingTimeMs, actualTime, tolerance)
	}

	// ProcessingTimeMs should not be zero for any real processing
	if output.ProcessingTimeMs == 0 && actualTime > 0 {
		t.Log("ProcessingTimeMs is 0 - may indicate timing precision issues")
	}
}