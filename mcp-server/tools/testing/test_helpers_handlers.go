package testing

import (
	"context"
	"encoding/json"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// TestHelpers integration handlers - these replace TypeScript TestHelpers functionality

// ServerStartHandler handles starting test servers
type ServerStartHandler struct{}

func NewServerStartHandler() *ServerStartHandler {
	return &ServerStartHandler{}
}

func (h *ServerStartHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ServerStartInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := StartServer(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// ServerStopHandler handles stopping test servers  
type ServerStopHandler struct{}

func NewServerStopHandler() *ServerStopHandler {
	return &ServerStopHandler{}
}

func (h *ServerStopHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ServerStopInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := StopServer(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// ServerStatusHandler handles checking server status
type ServerStatusHandler struct{}

func NewServerStatusHandler() *ServerStatusHandler {
	return &ServerStatusHandler{}
}

func (h *ServerStatusHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ServerStatusInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := GetServerStatus(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// TestProjectSetupHandler handles test project setup (replaces TestHelpers.resetTestProjectsFile)
type TestProjectSetupHandler struct{}

func NewTestProjectSetupHandler() *TestProjectSetupHandler {
	return &TestProjectSetupHandler{}
}

func (h *TestProjectSetupHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.TestProjectSetupInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := SetupTestProjects(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// EnsureTestDirectoryHandler handles test directory creation (replaces TestHelpers.ensureTestDirectory)
type EnsureTestDirectoryHandler struct{}

func NewEnsureTestDirectoryHandler() *EnsureTestDirectoryHandler {
	return &EnsureTestDirectoryHandler{}
}

func (h *EnsureTestDirectoryHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.EnsureTestDirectoryInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := EnsureTestDirectory(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// GenerateTestProjectNameHandler handles test project name generation (replaces TestHelpers.generateTestProjectName)
type GenerateTestProjectNameHandler struct{}

func NewGenerateTestProjectNameHandler() *GenerateTestProjectNameHandler {
	return &GenerateTestProjectNameHandler{}
}

func (h *GenerateTestProjectNameHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.GenerateTestProjectNameInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := GenerateTestProjectName(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// CleanupTestProjectsHandler handles comprehensive test cleanup (replaces TestHelpers.cleanupTestData)
type CleanupTestProjectsHandler struct{}

func NewCleanupTestProjectsHandler() *CleanupTestProjectsHandler {
	return &CleanupTestProjectsHandler{}
}

func (h *CleanupTestProjectsHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.CleanupTestProjectsInput]) (*mcp.CallToolResultFor[struct{}], error) {
	argsJSON, err := json.Marshal(params.Arguments)
	if err != nil {
		return nil, err
	}

	result, err := CleanupTestProjects(ctx, argsJSON)
	if err != nil {
		return nil, err
	}

	responseJSON, err := json.Marshal(result)
	if err != nil {
		return nil, err
	}

	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}