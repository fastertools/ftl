package mcp

import (
	"context"
	"fmt"
	"io"
	"log"
	"os/exec"
	"regexp"
	"strings"

	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// Validation functions
func validateProjectName(name string) error {
	if len(name) == 0 || len(name) > 50 {
		return fmt.Errorf("project name must be 1-50 characters, got %d", len(name))
	}
	
	// Allow alphanumeric, hyphens, underscores only
	if !regexp.MustCompile(`^[a-zA-Z0-9_-]+$`).MatchString(name) {
		return fmt.Errorf("project name contains invalid characters (allowed: a-z, A-Z, 0-9, _, -)")
	}
	
	// Prevent path traversal patterns
	if strings.Contains(name, "..") {
		return fmt.Errorf("project name cannot contain '..' sequences")
	}
	
	return nil
}

func validateTemplateName(template string) error {
	if template == "" {
		return nil // Empty is ok, defaults to "rust"
	}
	
	// Basic sanitization - no path traversal or shell chars
	if strings.ContainsAny(template, "../\\|<>&;$`()") {
		return fmt.Errorf("template name contains invalid characters")
	}
	
	// Length check
	if len(template) > 50 {
		return fmt.Errorf("template name too long (max 50 chars)")
	}
	
	return nil
}

func validateLinesCount(lines int) error {
	if lines < 0 {
		return fmt.Errorf("lines count cannot be negative")
	}
	
	if lines > 10000 {
		return fmt.Errorf("lines count cannot exceed 10000")
	}
	
	return nil
}

// Server represents an MCP server that exposes FTL functionality
type Server struct {
	server   *mcp.Server
	registry *ToolRegistry
}

// NewServer creates a new MCP server
func NewServer() *Server {
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "ftl-mcp-server",
		Version: "1.0.0",
	}, nil)
	
	return &Server{
		server:   server,
		registry: NewToolRegistry(),
	}
}

// RegisterTools registers all FTL tools with the MCP server
func (s *Server) RegisterTools(ctx context.Context) error {
	// Register ftl-init tool
	mcp.AddTool(s.server, &mcp.Tool{
		Name:        "ftl-init",
		Description: "Initialize a new FTL project",
	}, s.handleFtlInit)
	
	// Register ftl-build tool
	mcp.AddTool(s.server, &mcp.Tool{
		Name:        "ftl-build", 
		Description: "Build the current FTL project",
	}, s.handleFtlBuild)
	
	// Register ftl-up tool
	mcp.AddTool(s.server, &mcp.Tool{
		Name:        "ftl-up",
		Description: "Start the FTL development server", 
	}, s.handleFtlUp)
	
	// Register ftl-status tool
	mcp.AddTool(s.server, &mcp.Tool{
		Name:        "ftl-status",
		Description: "Get the current status of FTL applications",
	}, s.handleFtlStatus)
	
	// Register ftl-logs tool
	mcp.AddTool(s.server, &mcp.Tool{
		Name:        "ftl-logs",
		Description: "Get logs from FTL applications",
	}, s.handleFtlLogs)
	
	log.Printf("Registered 5 FTL tools")
	return nil
}

// Run starts the MCP server
func (s *Server) Run(ctx context.Context, stdin io.Reader, stdout io.Writer) error {
	return s.server.Run(ctx, &mcp.StdioTransport{})
}

// Tool handlers

// InitParams represents parameters for ftl-init
type InitParams struct {
	Name     string `json:"name"`
	Template string `json:"template,omitempty"`
}

// InitResult represents the output of ftl-init
type InitResult struct {
	Message string `json:"message"`
	Success bool   `json:"success"`
}

func (s *Server) handleFtlInit(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[InitParams]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments
	
	// Validate project name
	if err := validateProjectName(args.Name); err != nil {
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Invalid project name: %v", err)}},
		}, nil
	}
	
	// Validate template
	if err := validateTemplateName(args.Template); err != nil {
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Invalid template: %v", err)}},
		}, nil
	}
	
	template := "rust"
	if args.Template != "" {
		template = args.Template
	}
	
	cmd := exec.CommandContext(ctx, "ftl", "init", args.Name, "--template", template)
	output, err := cmd.CombinedOutput()
	
	result := InitResult{
		Success: err == nil,
		Message: fmt.Sprintf("Successfully initialized FTL project '%s' with %s template\n%s", args.Name, template, output),
	}
	
	if err != nil {
		result.Message = fmt.Sprintf("Error executing ftl init: %v\nOutput: %s", err, output)
	}
	
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: result.Message}},
	}, nil
}

// BuildParams represents parameters for ftl-build
type BuildParams struct {
	Watch bool `json:"watch,omitempty"`
}

// BuildResult represents the output of ftl-build
type BuildResult struct {
	Message string `json:"message"`
	Success bool   `json:"success"`
}

func (s *Server) handleFtlBuild(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[BuildParams]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments
	cmdArgs := []string{"build"}
	if args.Watch {
		cmdArgs = append(cmdArgs, "--watch")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	
	result := BuildResult{
		Success: err == nil,
		Message: fmt.Sprintf("Build completed successfully\n%s", output),
	}
	
	if err != nil {
		result.Message = fmt.Sprintf("Error executing ftl build: %v\nOutput: %s", err, output)
	}
	
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: result.Message}},
	}, nil
}

// UpParams represents parameters for ftl-up
type UpParams struct {
	Watch bool `json:"watch,omitempty"`
}

// UpResult represents the output of ftl-up
type UpResult struct {
	Message string `json:"message"`
	Success bool   `json:"success"`
}

func (s *Server) handleFtlUp(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[UpParams]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments
	cmdArgs := []string{"up"}
	if args.Watch {
		cmdArgs = append(cmdArgs, "--watch")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	
	result := UpResult{
		Success: err == nil,
		Message: fmt.Sprintf("Development server started\n%s", output),
	}
	
	if err != nil {
		result.Message = fmt.Sprintf("Error executing ftl up: %v\nOutput: %s", err, output)
	}
	
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: result.Message}},
	}, nil
}

// StatusParams represents parameters for ftl-status (no parameters needed)
type StatusParams struct{}

// StatusResult represents the output of ftl-status
type StatusResult struct {
	Message string `json:"message"`
	Success bool   `json:"success"`
}

func (s *Server) handleFtlStatus(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[StatusParams]) (*mcp.CallToolResultFor[struct{}], error) {
	cmd := exec.CommandContext(ctx, "ftl", "status")
	output, err := cmd.CombinedOutput()
	
	result := StatusResult{
		Success: err == nil,
		Message: string(output),
	}
	
	if err != nil {
		result.Message = fmt.Sprintf("Error executing ftl status: %v\nOutput: %s", err, output)
	}
	
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: result.Message}},
	}, nil
}

// LogsParams represents parameters for ftl-logs
type LogsParams struct {
	Follow bool `json:"follow,omitempty"`
	Lines  int  `json:"lines,omitempty"`
}

// LogsResult represents the output of ftl-logs
type LogsResult struct {
	Message string `json:"message"`
	Success bool   `json:"success"`
}

func (s *Server) handleFtlLogs(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[LogsParams]) (*mcp.CallToolResultFor[struct{}], error) {
	args := params.Arguments
	
	// Validate lines count
	if err := validateLinesCount(args.Lines); err != nil {
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: fmt.Sprintf("Invalid lines parameter: %v", err)}},
		}, nil
	}
	
	cmdArgs := []string{"logs"}
	
	if args.Lines > 0 {
		cmdArgs = append(cmdArgs, "--lines", fmt.Sprintf("%d", args.Lines))
	}
	
	if args.Follow {
		cmdArgs = append(cmdArgs, "--follow")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	
	result := LogsResult{
		Success: err == nil,
		Message: string(output),
	}
	
	if err != nil {
		result.Message = fmt.Sprintf("Error executing ftl logs: %v\nOutput: %s", err, output)
	}
	
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: result.Message}},
	}, nil
}

