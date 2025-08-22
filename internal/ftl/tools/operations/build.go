package operations

import (
	"context"
	"encoding/json"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/ftl/ftl"
	"github.com/fastertools/ftl/internal/ftl/types"
)

// BuildHandler handles ftl build operations
type BuildHandler struct {
	ftlCommander *ftl.Commander
}

// NewBuildHandler creates a new build handler
func NewBuildHandler(ftlCommander *ftl.Commander) *BuildHandler {
	return &BuildHandler{
		ftlCommander: ftlCommander,
	}
}

// Handle processes the ftl build request
func (h *BuildHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.FTLBuildInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: ftlBuild function called\n")
	projectPath := params.Arguments.ProjectPath
	clean := params.Arguments.Clean

	// Log the incoming payload to stderr
	fmt.Fprintf(os.Stderr, "DEBUG: ftl_build received payload: project_path='%s', clean='%t'\n", projectPath, clean)

	// Validate project directory
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		response := types.BuildResponse{
			Success:     false,
			ProjectPath: projectPath,
			Error:       err.Error(),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Execute build command
	output, err := h.ftlCommander.ExecuteBuild(projectPath, clean)
	
	if err != nil {
		response := types.BuildResponse{
			Success:     false,
			Output:      output,
			ProjectPath: projectPath,
			Error:       fmt.Sprintf("Build failed: %s", err),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	response := types.BuildResponse{
		Success:     true,
		Output:      output,
		ProjectPath: projectPath,
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}