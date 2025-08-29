package operations

import (
	"context"
	"encoding/json"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/types"
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
	
	// Extract options from embedded struct or legacy fields
	var clean bool
	var skipSynth bool
	var configFile string
	
	if params.Arguments.BuildOptions != nil {
		// Use new embedded options
		clean = false // Clean option not available in BuildOptions, keep as legacy
		skipSynth = params.Arguments.BuildOptions.SkipSynth
		configFile = params.Arguments.BuildOptions.ConfigFile
	}
	
	// Always check legacy field for clean (not part of CLI BuildOptions)
	if params.Arguments.Clean {
		clean = true
	}

	// Log the incoming payload to stderr
	fmt.Fprintf(os.Stderr, "DEBUG: ftl_build received payload: project_path='%s', clean='%t', skip_synth='%t', config_file='%s'\n", 
		projectPath, clean, skipSynth, configFile)

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
	// TODO: Update ftl.Commander to support full BuildOptions (SkipSynth, ConfigFile)
	// For now, only clean flag is supported
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