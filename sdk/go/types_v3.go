// Package ftl - V3 Type Definitions
//
// This file defines additional types specific to the V3 API that enhance
// the existing types with type safety and Go idiomaticity.
package ftl

import (
	"context"
	"fmt"
	"regexp"
	"strings"
	"sync"
	"time"
)

// V3 API Version constant
const (
	FTLSDKVersionV3 = "v3"
)

// ToolContext provides request-specific context for V3 handlers.
// This extends context.Context with tool-specific information.
type ToolContext struct {
	context.Context
	
	// ToolName is the name of the tool being executed
	ToolName string
	
	// RequestID is a unique identifier for this request (for debugging/tracing)
	RequestID string
	
	// StartTime is when the request started processing
	StartTime time.Time
}

// NewToolContext creates a context for tool execution.
// This will be used by the V3 handler wrapper to provide enhanced context.
func NewToolContext(toolName string) *ToolContext {
	return &ToolContext{
		Context:   context.Background(),
		ToolName:  toolName,
		RequestID: generateRequestID(),
		StartTime: time.Now(),
	}
}

// WithTimeout adds a timeout to the tool context.
func (tc *ToolContext) WithTimeout(duration time.Duration) (context.Context, context.CancelFunc) {
	return context.WithTimeout(tc.Context, duration)
}

// WithCancel adds cancellation capability to the tool context.
func (tc *ToolContext) WithCancel() (context.Context, context.CancelFunc) {
	return context.WithCancel(tc.Context)
}

// Log provides structured logging with tool context.
// This preserves the existing security-aware logging while adding context.
func (tc *ToolContext) Log(level string, message string, args ...interface{}) {
	// Use existing secure logging with context prefix
	prefix := fmt.Sprintf("[%s:%s] %s", level, tc.ToolName, message)
	secureLogf(prefix, args...)
}

// ValidationError represents input validation errors in V3 handlers.
type ValidationError struct {
	Field   string
	Message string
}

func (e ValidationError) Error() string {
	return fmt.Sprintf("validation failed for field '%s': %s", e.Field, e.Message)
}

// ToolError represents execution errors in V3 handlers.
type ToolError struct {
	Code    string
	Message string
	Cause   error
}

func (e ToolError) Error() string {
	sanitizedMessage := sanitizeErrorMessage(e.Message)
	if e.Cause != nil {
		// If the message was already sanitized to generic, don't add more text
		if sanitizedMessage == "An error occurred during processing" {
			return sanitizedMessage
		}
		// Don't expose underlying cause details in string representation
		return fmt.Sprintf("%s: internal error occurred", sanitizedMessage)
	}
	return sanitizedMessage
}

func (e ToolError) Unwrap() error {
	return e.Cause
}

// Common error constructors for V3 handlers

// InvalidInput creates a validation error for invalid input fields.
func InvalidInput(field, message string) error {
	return ValidationError{Field: field, Message: message}
}

// ToolFailed creates a tool execution error.
func ToolFailed(message string, cause error) error {
	return ToolError{Code: "execution_failed", Message: message, Cause: cause}
}

// InternalError creates an internal server error.
func InternalError(message string) error {
	return ToolError{Code: "internal_error", Message: message, Cause: nil}
}

// NewToolError creates a custom tool error with specific code.
func NewToolError(code, message string) error {
	return ToolError{Code: code, Message: message, Cause: nil}
}

// TypedToolDefinition represents a V3 tool definition with type information.
// This extends the base ToolDefinition with V3-specific metadata.
type TypedToolDefinition struct {
	ToolDefinition
	
	// InputType is the Go type name for input (for documentation/debugging)
	InputType string
	
	// OutputType is the Go type name for output (for documentation/debugging)  
	OutputType string
	
	// SchemaGenerated indicates if the schema was auto-generated
	SchemaGenerated bool
}

// V3ToolRegistry manages V3 tool registrations.
// This is used internally to track V3-specific tool metadata.
type V3ToolRegistry struct {
	mu               sync.RWMutex
	tools            map[string]TypedToolDefinition
	registeredV3Tools map[string]bool // Moved inside registry for thread safety
}

// Global V3 registry (for internal use)
var v3Registry = &V3ToolRegistry{
	tools:            make(map[string]TypedToolDefinition),
	registeredV3Tools: make(map[string]bool),
}

// RegisterTypedTool adds a tool to the V3 registry (internal use).
func (r *V3ToolRegistry) RegisterTypedTool(name string, definition TypedToolDefinition) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.tools[name] = definition
	r.registeredV3Tools[name] = true
}

// GetTypedTool retrieves a tool from the V3 registry (internal use).
func (r *V3ToolRegistry) GetTypedTool(name string) (TypedToolDefinition, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	tool, exists := r.tools[name]
	return tool, exists
}

// GetAllTypedTools returns all registered V3 tools (for debugging).
func (r *V3ToolRegistry) GetAllTypedTools() map[string]TypedToolDefinition {
	r.mu.RLock()
	defer r.mu.RUnlock()
	// Return a copy to prevent modification
	result := make(map[string]TypedToolDefinition)
	for k, v := range r.tools {
		result[k] = v
	}
	return result
}

// IsV3ToolRegistered checks if a tool was registered via V3 API (for testing).
func (r *V3ToolRegistry) IsV3ToolRegistered(name string) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()
	return r.registeredV3Tools[name]
}

// ClearV3Tools clears all V3 tool registrations (for testing).
func (r *V3ToolRegistry) ClearV3Tools() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.tools = make(map[string]TypedToolDefinition)
	r.registeredV3Tools = make(map[string]bool)
}

// Helper functions

// generateRequestID creates a unique request ID for debugging/tracing.
// CRAWL phase: Simple implementation, RUN phase will use more sophisticated approach.
func generateRequestID() string {
	return fmt.Sprintf("req_%d", time.Now().UnixNano()%1000000)
}

// sanitizeErrorMessage sanitizes error messages to prevent information disclosure.
// Uses a hybrid approach: removes known-sensitive patterns but allows most other messages.
func sanitizeErrorMessage(msg string) string {
	// First, check for empty or whitespace-only messages
	if msg == "" || len(strings.TrimSpace(msg)) == 0 {
		return "An error occurred during processing"
	}
	
	// Check for definitely sensitive patterns that should never be exposed
	sensitivePatterns := []string{
		`/[a-zA-Z0-9._/-]+\.go:\d+`,     // Go source file references
		`panic:`,                         // Panic stack traces  
		`runtime\.`,                      // Runtime internals
		`reflect\.`,                      // Reflection internals
		`0x[0-9a-fA-F]{8,}`,             // Memory addresses (8+ hex digits)
		`goroutine \d+`,                  // Goroutine information
		`\(\*[a-zA-Z]+\)`,               // Go type information
		`/Users/[^\s]+`,                  // User paths
		`/home/[^\s]+`,                   // Home paths
		`[A-Z]:\\\\`,                      // Windows paths
	}
	
	for _, pattern := range sensitivePatterns {
		if matched, _ := regexp.MatchString(pattern, msg); matched {
			// Message contains sensitive information
			return "An error occurred during processing"
		}
	}
	
	// If message is too long, truncate it
	if len(msg) > 200 {
		return msg[:200] + "..."
	}
	
	// Message appears safe, return as-is
	return msg
}

// convertError converts Go errors to ToolResponse for V3 handlers.
// This preserves existing security features while providing better error handling.
func convertError(err error) ToolResponse {
	if err == nil {
		// Return empty success response
		return ToolResponse{Content: []ToolContent{}}
	}
	
	// Handle specific error types
	switch e := err.(type) {
	case ValidationError:
		// ValidationError messages are user-facing and safe to expose
		return Error(fmt.Sprintf("Invalid input for field '%s': %s", e.Field, sanitizeErrorMessage(e.Message)))
	case ToolError:
		// ToolError messages may contain sensitive information from underlying causes
		sanitizedMessage := sanitizeErrorMessage(e.Message)
		if e.Cause != nil {
			// Don't expose the underlying cause details - they may contain sensitive info
			return Error(fmt.Sprintf("%s: internal error occurred", sanitizedMessage))
		}
		return Error(sanitizedMessage)
	default:
		// For unknown errors, provide minimal information to prevent leakage
		return Error("An error occurred during processing")
	}
}

// Serve starts the HTTP server for V3 tools (convenience function).
// This wraps the existing serving mechanism with V3 semantics.
func Serve() {
	secureLogf("V3 FTL SDK server ready with %d tools registered", len(v3Registry.tools))
	
	// Convert V3 tools to legacy format for compatibility
	legacyTools := make(map[string]ToolDefinition)
	for name, typedDef := range v3Registry.tools {
		legacyTools[name] = typedDef.ToolDefinition
	}
	
	// Use existing CreateTools infrastructure (if available)
	createToolsIfAvailable(legacyTools)
	
	secureLogf("FTL SDK HTTP server started successfully")
}

// GetV3APIInfo returns information about the V3 API for debugging/introspection.
func GetV3APIInfo() map[string]interface{} {
	return map[string]interface{}{
		"version":     FTLSDKVersionV3,
		"tools_count": len(v3Registry.tools),
		"features": []string{
			"type_safe_handlers",
			"automatic_schema_generation",
			"enhanced_response_building",
			"context_support",
			"structured_error_handling",
		},
	}
}