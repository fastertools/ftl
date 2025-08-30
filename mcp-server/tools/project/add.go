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

// AddHandler handles adding tools to FTL projects
type AddHandler struct {
	ftlCommander *ftl.Commander
}

// NewAddHandler creates a new add handler
func NewAddHandler(ftlCommander *ftl.Commander) *AddHandler {
	return &AddHandler{
		ftlCommander: ftlCommander,
	}
}

// AddToolResponse represents the result of adding a tool to a project
type AddToolResponse struct {
	ProjectPath   string   `json:"project_path"`
	ToolName      string   `json:"tool_name"`
	Registry      string   `json:"registry"`
	Version       string   `json:"version"`
	Language      string   `json:"language"`
	FilesCreated  []string `json:"files_created"`
	FilesModified []string `json:"files_modified"`
	Success       bool     `json:"success"`
	Message       string   `json:"message,omitempty"`
	Error         string   `json:"error,omitempty"`
}

// Handle processes the add tool request
func (h *AddHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.AddToolInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: addTool function called\n")
	input := params.Arguments
	// Determine project path
	var projectPath string
	if input.ProjectPath != "" {
		projectPath = input.ProjectPath
	} else if input.ProjectName != "" {
		// If only project name given, assume current directory or standard location
		if _, err := os.Stat(input.ProjectName); err == nil {
			projectPath = input.ProjectName
		} else {
			projectPath = filepath.Join(".", input.ProjectName)
		}
	} else {
		projectPath = "." // Default to current directory
	}

	// Verify project exists
	if _, err := os.Stat(projectPath); os.IsNotExist(err) {
			response := AddToolResponse{
			ProjectPath: projectPath,
			ToolName:    input.ToolName,
			Success:     false,
			Error:       fmt.Sprintf("Project directory does not exist: %s", projectPath),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Set defaults
	if input.Registry == "" {
		input.Registry = "ferm" // Use default registry
	}
	if input.Version == "" {
		input.Version = "latest"
	}

	// Build ftl add command
	args := []string{"add", input.ToolName}
	
	if input.Language != "" {
		args = append(args, "--language", input.Language)
	}
	if input.Registry != "ferm" { // Only specify if not default
		args = append(args, "--registry", input.Registry)
	}
	if input.Version != "latest" { // Only specify if not default
		args = append(args, "--version", input.Version)
	}
	if input.Yes {
		args = append(args, "--yes")
	}
	
	// Use --project-path flag to specify target directory
	args = append(args, "--project-path", projectPath)

	// Capture files before operation (for comparison)
	filesBefore := make(map[string]os.FileInfo)
	filepath.Walk(projectPath, func(path string, info os.FileInfo, err error) error {
		if err == nil && !info.IsDir() {
			filesBefore[path] = info
		}
		return nil
	})

	// Execute ftl add command
	cmd := exec.Command(h.ftlCommander.GetExecutable(), args...)
	
	fmt.Fprintf(os.Stderr, "DEBUG: Executing command: %s %s in directory: %s\n",
		h.ftlCommander.GetExecutable(), strings.Join(args, " "), projectPath)
	
	output, err := cmd.CombinedOutput()
	outputStr := string(output)
	
	fmt.Fprintf(os.Stderr, "DEBUG: Command output: %s\n", outputStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "DEBUG: Command error: %v\n", err)
		response := AddToolResponse{
			ProjectPath: projectPath,
			ToolName:    input.ToolName,
			Registry:    input.Registry,
			Version:     input.Version,
			Language:    input.Language,
			Success:     false,
			Error:       fmt.Sprintf("Failed to add tool: %s", err.Error()),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Determine what files were created/modified
	var filesCreated, filesModified []string
	filepath.Walk(projectPath, func(path string, info os.FileInfo, err error) error {
		if err != nil || info.IsDir() {
			return err
		}
		
		if beforeInfo, existed := filesBefore[path]; existed {
			// File existed - check if modified
			if !beforeInfo.ModTime().Equal(info.ModTime()) || beforeInfo.Size() != info.Size() {
				if relPath, err := filepath.Rel(projectPath, path); err == nil {
					filesModified = append(filesModified, relPath)
				}
			}
		} else {
			// New file
			if relPath, err := filepath.Rel(projectPath, path); err == nil {
				filesCreated = append(filesCreated, relPath)
			}
		}
		return nil
	})

	response := AddToolResponse{
		ProjectPath:   projectPath,
		ToolName:      input.ToolName,
		Registry:      input.Registry,
		Version:       input.Version,
		Language:      input.Language,
		FilesCreated:  filesCreated,
		FilesModified: filesModified,
		Success:       true,
		Message:       fmt.Sprintf("Successfully added tool '%s'", input.ToolName),
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

