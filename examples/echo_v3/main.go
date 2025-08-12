package main

import (
	"context"
	"fmt"
	"strings"
	"time"

	ftl "github.com/fastertools/ftl-cli/sdk/go"
)

// EchoInput represents the input structure for the echo tool.
// This demonstrates how V3 handlers use struct tags for automatic
// JSON schema generation.
type EchoInput struct {
	// Message is the text to echo back
	Message string `json:"message" jsonschema:"required,description=Message to echo back"`
	
	// Count specifies how many times to repeat the message (optional)
	Count int `json:"count,omitempty" jsonschema:"minimum=1,maximum=10,description=Number of times to repeat the message"`
	
	// Prefix is an optional prefix for each repetition
	Prefix string `json:"prefix,omitempty" jsonschema:"description=Optional prefix for each message repetition"`
}

// EchoOutput represents the output structure for the echo tool.
// This demonstrates type-safe output handling in V3.
type EchoOutput struct {
	// Response contains the echoed message(s)
	Response string `json:"response"`
	
	// EchoedAt contains the timestamp when the echo was processed
	EchoedAt string `json:"echoed_at"`
	
	// RepetitionCount shows how many times the message was repeated
	RepetitionCount int `json:"repetition_count"`
	
	// ProcessingTimeMs shows how long the operation took
	ProcessingTimeMs int64 `json:"processing_time_ms"`
}

// EchoHandler demonstrates a V3 type-safe handler.
// It takes typed input and returns typed output with proper error handling.
func EchoHandler(ctx context.Context, input EchoInput) (EchoOutput, error) {
	startTime := time.Now()
	
	// Validate input
	if input.Message == "" {
		return EchoOutput{}, ftl.InvalidInput("message", "message cannot be empty")
	}
	
	// Set default count if not specified
	count := input.Count
	if count <= 0 {
		count = 1
	}
	
	// Build the response
	var responses []string
	for i := 0; i < count; i++ {
		msg := input.Message
		if input.Prefix != "" {
			msg = input.Prefix + msg
		}
		responses = append(responses, msg)
	}
	
	// Calculate processing time
	processingTime := time.Since(startTime).Milliseconds()
	
	return EchoOutput{
		Response:         strings.Join(responses, "\n"),
		EchoedAt:        time.Now().Format(time.RFC3339),
		RepetitionCount:  count,
		ProcessingTimeMs: processingTime,
	}, nil
}

// GreetingHandler demonstrates another V3 handler with different types
func GreetingHandler(ctx context.Context, input GreetingInput) (GreetingOutput, error) {
	// Simple validation
	if input.Name == "" {
		return GreetingOutput{}, ftl.InvalidInput("name", "name is required")
	}
	
	// Create personalized greeting
	greeting := fmt.Sprintf("Hello, %s!", input.Name)
	if input.Formal {
		greeting = fmt.Sprintf("Good day, %s. How may I assist you today?", input.Name)
	}
	
	return GreetingOutput{
		Greeting: greeting,
		Language: "en",
		Formal:   input.Formal,
	}, nil
}

// GreetingInput for the greeting tool
type GreetingInput struct {
	Name   string `json:"name" jsonschema:"required,description=Name of the person to greet"`
	Formal bool   `json:"formal,omitempty" jsonschema:"description=Use formal greeting style"`
}

// GreetingOutput for the greeting tool  
type GreetingOutput struct {
	Greeting string `json:"greeting"`
	Language string `json:"language"`
	Formal   bool   `json:"formal"`
}

// init registers the V3 tools using the new type-safe API
func init() {
	// Register the echo handler
	ftl.HandleTypedTool("echo", EchoHandler)
	
	// Register the greeting handler to show multiple tools
	ftl.HandleTypedTool("greeting", GreetingHandler)
}

// main is required by Spin but doesn't need to do anything
func main() {
	// Spin handles the HTTP server lifecycle
	// The init() function already registered our tools
}