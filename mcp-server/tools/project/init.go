package project

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/ftl"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// InitHandler handles project initialization
type InitHandler struct {
	ftlCommander *ftl.Commander
}

// NewInitHandler creates a new init handler
func NewInitHandler(ftlCommander *ftl.Commander) *InitHandler {
	return &InitHandler{
		ftlCommander: ftlCommander,
	}
}

// InitProjectResponse represents the result of project initialization
type InitProjectResponse struct {
	ProjectPath   string   `json:"project_path"`
	ProjectName   string   `json:"project_name"`
	Language      string   `json:"language"`
	Template      string   `json:"template"`
	BasePath      string   `json:"base_path"`
	FilesCreated  []string `json:"files_created"`
	Success       bool     `json:"success"`
	Message       string   `json:"message,omitempty"`
	Error         string   `json:"error,omitempty"`
}

// Handle processes the project init request
func (h *InitHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.InitProjectInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: initProject function called\n")
	input := params.Arguments
	
	// Set defaults
	if input.Language == "" {
		input.Language = "rust"
	}
	if input.BasePath == "" {
		input.BasePath = "."
	}
	if input.Template == "" {
		input.Template = input.Language // Use language as default template
	}

	// Construct project path
	projectPath := filepath.Join(input.BasePath, input.ProjectName)
	
	fmt.Fprintf(os.Stderr, "DEBUG: init_project received payload: project_name='%s', language='%s', base_path='%s', template='%s'\n",
		input.ProjectName, input.Language, input.BasePath, input.Template)

	// Check if project already exists
	if _, err := os.Stat(projectPath); err == nil {
		response := InitProjectResponse{
			ProjectPath: projectPath,
			ProjectName: input.ProjectName,
			Language:    input.Language,
			Template:    input.Template,
			BasePath:    input.BasePath,
			Success:     false,
			Error:       fmt.Sprintf("Project directory already exists: %s", projectPath),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Build ftl init command
	args := []string{"init", input.ProjectName, "--language", "yaml"} // Use yaml as config language
	if input.Template != "" {
		args = append(args, "--template", input.Template)
	} else {
		args = append(args, "--template", "mcp") // Default to mcp template
	}

	// Execute ftl init command
	cmd := exec.Command(h.ftlCommander.GetExecutable(), args...)
	cmd.Dir = input.BasePath
	
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %s in directory: %s\n",
		h.ftlCommander.GetExecutable(), strings.Join(args, " "), input.BasePath)
	
	output, err := cmd.CombinedOutput()
	outputStr := string(output)
	
	fmt.Fprintf(os.Stderr, "DEBUG: Command output: %s\n", outputStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Command error: %v\n", err)
		response := InitProjectResponse{
			ProjectPath: projectPath,
			ProjectName: input.ProjectName,
			Language:    input.Language,
			Template:    input.Template,
			BasePath:    input.BasePath,
			Success:     false,
			Error:       fmt.Sprintf("Failed to initialize project: %s", err.Error()),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Get list of created files
	filesCreated := []string{}
	if _, err := os.Stat(projectPath); err == nil {
		filepath.Walk(projectPath, func(path string, info os.FileInfo, err error) error {
			if err != nil || info.IsDir() {
				return err
			}
			relPath, _ := filepath.Rel(projectPath, path)
			filesCreated = append(filesCreated, relPath)
			return nil
		})
	}

	response := InitProjectResponse{
		ProjectPath:  projectPath,
		ProjectName:  input.ProjectName,
		Language:     input.Language,
		Template:     input.Template,
		BasePath:     input.BasePath,
		FilesCreated: filesCreated,
		Success:      true,
		Message:      fmt.Sprintf("Successfully initialized %s project '%s'", input.Language, input.ProjectName),
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

