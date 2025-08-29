package testing

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// Handler provides MCP tool interface for enhanced testing functionality
type Handler struct {
	testHelpers *TestHelpers
}

// NewHandler creates a new testing handler
func NewHandler(testMode bool) *Handler {
	return &Handler{
		testHelpers: NewTestHelpers(testMode),
	}
}

// HandleProjectLifecycleTest handles the project lifecycle test MCP tool
func (h *Handler) HandleProjectLifecycleTest(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ProjectLifecycleTestInput]) (*mcp.CallToolResultFor[ProjectLifecycleResult], error) {
	input := params.Arguments

	// Create test context from input
	testContext := TestContext{
		ProjectName: input.ProjectName,
		ProjectPath: input.ProjectPath,
		Language:    input.Language,
		TestID:      input.TestID,
		TempDir:     input.TempDir,
	}

	// Override with defaults if not provided
	if testContext.Language == "" {
		testContext.Language = "rust"
	}
	if testContext.TempDir == "" {
		testContext.TempDir = h.testHelpers.tempDir
	}
	if testContext.ProjectPath == "" {
		testContext.ProjectPath = fmt.Sprintf("%s/%s", testContext.TempDir, testContext.ProjectName)
	}

	// Run project lifecycle test
	result, err := h.testHelpers.TestProjectLifecycle(ctx, testContext)
	if err != nil {
		return &mcp.CallToolResultFor[ProjectLifecycleResult]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Project lifecycle test failed: %v", err)}},
		}, nil // Don't return error, return failure result instead
	}

	// Return success result
	resultJSON, _ := json.Marshal(result)
	return &mcp.CallToolResultFor[ProjectLifecycleResult]{
		Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Project lifecycle test completed: %s", string(resultJSON))}},
	}, nil
}

// HandleMCPToolTest handles testing of individual MCP tools
func (h *Handler) HandleMCPToolTest(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.MCPToolTestInput]) (*mcp.CallToolResultFor[MCPToolResult], error) {
	input := params.Arguments

	// Parse tool input from JSON
	var toolInput interface{}
	if err := json.Unmarshal([]byte(input.ToolInput), &toolInput); err != nil {
		return &mcp.CallToolResultFor[MCPToolResult]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Failed to parse tool input: %v", err)}},
		}, nil
	}

	// Run MCP tool test
	result, err := h.testHelpers.TestMCPTool(ctx, input.ToolName, toolInput, input.ExpectedFiles)
	if err != nil {
		return &mcp.CallToolResultFor[MCPToolResult]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("MCP tool test failed: %v", err)}},
		}, nil
	}

	// Return result
	resultJSON, _ := json.Marshal(result)
	return &mcp.CallToolResultFor[MCPToolResult]{
		Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("MCP tool test completed: %s", string(resultJSON))}},
	}, nil
}

// HandleCleanup handles cleanup of test resources
func (h *Handler) HandleCleanup(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[struct{}]) (*mcp.CallToolResultFor[struct{}], error) {
	err := h.testHelpers.Cleanup()
	if err != nil {
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Cleanup failed: %v", err)}},
		}, nil
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: "Test cleanup completed successfully"}},
	}, nil
}