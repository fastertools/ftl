package mcp

import (
	"context"
	"testing"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewServer(t *testing.T) {
	server := NewServer()
	assert.NotNil(t, server)
	assert.NotNil(t, server.server)
	assert.NotNil(t, server.registry)
}

func TestRegisterTools(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	err := server.RegisterTools(ctx)
	require.NoError(t, err)
}

func TestFtlInitHandler(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	// Test successful init
	params := &mcp.CallToolParamsFor[InitParams]{
		Arguments: InitParams{
			Name:     "test-project",
			Template: "rust",
		},
	}
	
	result, err := server.handleFtlInit(ctx, nil, params)
	
	// Since we don't have ftl command available in test, expect an error
	// but the handler should still return a result
	assert.NotNil(t, result)
	assert.NoError(t, err) // Handler doesn't return the command error
	assert.NotEmpty(t, result.Content)
}

func TestFtlBuildHandler(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	params := &mcp.CallToolParamsFor[BuildParams]{
		Arguments: BuildParams{
			Watch: false,
		},
	}
	
	result, err := server.handleFtlBuild(ctx, nil, params)
	
	assert.NotNil(t, result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Content)
}

func TestFtlUpHandler(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	params := &mcp.CallToolParamsFor[UpParams]{
		Arguments: UpParams{
			Watch: true,
		},
	}
	
	result, err := server.handleFtlUp(ctx, nil, params)
	
	assert.NotNil(t, result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Content)
}

func TestFtlStatusHandler(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	params := &mcp.CallToolParamsFor[StatusParams]{
		Arguments: StatusParams{},
	}
	
	result, err := server.handleFtlStatus(ctx, nil, params)
	
	assert.NotNil(t, result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Content)
}

func TestFtlLogsHandler(t *testing.T) {
	server := NewServer()
	ctx := context.Background()
	
	params := &mcp.CallToolParamsFor[LogsParams]{
		Arguments: LogsParams{
			Follow: false,
			Lines:  10,
		},
	}
	
	result, err := server.handleFtlLogs(ctx, nil, params)
	
	assert.NotNil(t, result)
	assert.NoError(t, err)
	assert.NotEmpty(t, result.Content)
}