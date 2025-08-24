package testing

import (
	"context"
	"encoding/json"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// GetTestConfigHandler handles the get_test_config tool
type GetTestConfigHandler struct{}

// NewGetTestConfigHandler creates a new get test config handler
func NewGetTestConfigHandler() *GetTestConfigHandler {
	return &GetTestConfigHandler{}
}

// Handle executes the get_test_config operation
func (h *GetTestConfigHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.GetTestConfigInput]) (*mcp.CallToolResultFor[struct{}], error) {
	// Marshal the arguments to JSON
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	// Call the actual function
	result, err := GetTestConfig(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	// Convert result to JSON
	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// UpdateTestConfigHandler handles the update_test_config tool
type UpdateTestConfigHandler struct{}

// NewUpdateTestConfigHandler creates a new update test config handler
func NewUpdateTestConfigHandler() *UpdateTestConfigHandler {
	return &UpdateTestConfigHandler{}
}

// Handle executes the update_test_config operation
func (h *UpdateTestConfigHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.UpdateTestConfigInput]) (*mcp.CallToolResultFor[struct{}], error) {
	// Marshal the arguments to JSON
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	// Call the actual function
	result, err := UpdateTestConfig(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	// Convert result to JSON
	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// CreateTestProjectHandler handles the create_test_project tool
type CreateTestProjectHandler struct{}

// NewCreateTestProjectHandler creates a new create test project handler
func NewCreateTestProjectHandler() *CreateTestProjectHandler {
	return &CreateTestProjectHandler{}
}

// Handle executes the create_test_project operation
func (h *CreateTestProjectHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.CreateTestProjectInput]) (*mcp.CallToolResultFor[struct{}], error) {
	// Marshal the arguments to JSON
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	// Call the actual function
	result, err := CreateTestProject(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	// Convert result to JSON
	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// CleanupTestDataHandler handles the cleanup_test_data tool
type CleanupTestDataHandler struct{}

// NewCleanupTestDataHandler creates a new cleanup test data handler
func NewCleanupTestDataHandler() *CleanupTestDataHandler {
	return &CleanupTestDataHandler{}
}

// Handle executes the cleanup_test_data operation
func (h *CleanupTestDataHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.CleanupTestDataInput]) (*mcp.CallToolResultFor[struct{}], error) {
	// Marshal the arguments to JSON
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	// Call the actual function
	result, err := CleanupTestData(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	// Convert result to JSON
	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}