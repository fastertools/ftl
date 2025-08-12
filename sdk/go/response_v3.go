// Package ftl - V3 Response Builder
//
// This file provides enhanced response building APIs that follow Go's
// builder pattern conventions (similar to strings.Builder).
package ftl

import (
	"encoding/base64"
	"encoding/json"
)

// ResponseBuilder provides a fluent API for building ToolResponse objects.
// It follows Go's builder pattern conventions and makes it easy to construct
// complex responses with multiple content types.
//
// Example:
//
//	response := NewResponse().
//	    AddText("Processing completed successfully").
//	    AddStructured(result).
//	    Build()
type ResponseBuilder struct {
	contents []ToolContent
	isError  bool
	structured interface{}
}

// NewResponse creates a new response builder.
func NewResponse() *ResponseBuilder {
	return &ResponseBuilder{
		contents: make([]ToolContent, 0),
		isError:  false,
	}
}

// AddText adds text content to the response (chainable).
// This is the most common type of content for tool responses.
func (rb *ResponseBuilder) AddText(text string) *ResponseBuilder {
	rb.contents = append(rb.contents, ToolContent{
		Type: ContentTypeText,
		Text: text,
	})
	return rb
}

// AddTextf adds formatted text content to the response (chainable).
// Convenience method similar to fmt.Sprintf.
func (rb *ResponseBuilder) AddTextf(format string, args ...interface{}) *ResponseBuilder {
	return rb.AddText(Textf(format, args...).Content[0].Text)
}

// AddImage adds image content to the response (chainable).
// Data should be the raw image bytes, mimeType should be like "image/png".
func (rb *ResponseBuilder) AddImage(data []byte, mimeType string) *ResponseBuilder {
	rb.contents = append(rb.contents, ToolContent{
		Type:     ContentTypeImage,
		Data:     base64.StdEncoding.EncodeToString(data),
		MimeType: mimeType,
	})
	return rb
}

// AddAudio adds audio content to the response (chainable).
// Data should be the raw audio bytes, mimeType should be like "audio/wav".
func (rb *ResponseBuilder) AddAudio(data []byte, mimeType string) *ResponseBuilder {
	rb.contents = append(rb.contents, ToolContent{
		Type:     ContentTypeAudio,
		Data:     base64.StdEncoding.EncodeToString(data),
		MimeType: mimeType,
	})
	return rb
}

// AddResource adds resource content to the response (chainable).
// This is for referencing external resources.
func (rb *ResponseBuilder) AddResource(resource *ResourceContents) *ResponseBuilder {
	rb.contents = append(rb.contents, ToolContent{
		Type:     ContentTypeResource,
		Resource: resource,
	})
	return rb
}

// AddStructured adds structured data to the response (chainable).
// The data will be JSON marshaled and included as structured content.
// This is useful for providing machine-readable data alongside human-readable text.
func (rb *ResponseBuilder) AddStructured(data interface{}) *ResponseBuilder {
	// RUN phase: Make a deep copy to ensure immutability
	if data != nil {
		// Use JSON marshal/unmarshal for deep copy
		if jsonData, err := json.Marshal(data); err == nil {
			var copy interface{}
			if err := json.Unmarshal(jsonData, &copy); err == nil {
				rb.structured = copy
			} else {
				rb.structured = data // Fallback to original if copy fails
			}
		} else {
			rb.structured = data // Fallback to original if marshal fails
		}
	} else {
		rb.structured = data
	}
	
	return rb
}

// WithError marks the response as an error response.
// This affects how the gateway handles the response.
func (rb *ResponseBuilder) WithError() *ResponseBuilder {
	rb.isError = true
	return rb
}

// WithAnnotations adds annotations to the most recently added content.
// Returns the builder for chaining. If no content has been added, this is a no-op.
func (rb *ResponseBuilder) WithAnnotations(annotations *ContentAnnotations) *ResponseBuilder {
	if len(rb.contents) > 0 {
		rb.contents[len(rb.contents)-1].Annotations = annotations
	}
	return rb
}

// Build creates the final ToolResponse.
// This consumes the builder and returns the constructed response.
func (rb *ResponseBuilder) Build() ToolResponse {
	response := ToolResponse{
		Content:           rb.contents,
		IsError:           rb.isError,
		StructuredContent: rb.structured,
	}
	
	return response
}

// Helper functions for common response patterns

// TextResponse creates a simple text response (convenience function).
func TextResponse(text string) ToolResponse {
	return NewResponse().AddText(text).Build()
}

// ErrorResponse creates an error response with text (convenience function).
func ErrorResponse(errorText string) ToolResponse {
	return NewResponse().AddText(errorText).WithError().Build()
}

// StructuredResponse creates a response with both text and structured data (convenience function).
func StructuredResponse(text string, data interface{}) ToolResponse {
	return NewResponse().AddText(text).AddStructured(data).Build()
}

// typedResponse converts typed output to ToolResponse (internal helper).
// This is used by the V3 handler wrapper to convert typed handler output.
func typedResponse[T any](output T) ToolResponse {
	// RUN phase: Clean structured response with optional text representation
	if jsonData, err := json.MarshalIndent(output, "", "  "); err == nil {
		return NewResponse().AddText("Result:\n" + string(jsonData)).AddStructured(output).Build()
	}
	
	// Fallback if JSON marshaling fails
	return NewResponse().AddText("Result processing completed").AddStructured(output).Build()
}

// EmptyResponse creates an empty successful response (convenience function).
func EmptyResponse() ToolResponse {
	return ToolResponse{
		Content: []ToolContent{},
		IsError: false,
	}
}