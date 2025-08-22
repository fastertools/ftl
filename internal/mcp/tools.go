package mcp

import (
	"context"
	"fmt"
	"os/exec"
)

// InitTool implements the ftl init command
type InitTool struct{}

func (t *InitTool) Definition() ToolDefinition {
	return ToolDefinition{
		Name:        "ftl-init",
		Description: "Initialize a new FTL project",
		Schema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"name": map[string]interface{}{
					"type":        "string",
					"description": "Name of the project to create",
				},
				"template": map[string]interface{}{
					"type":        "string", 
					"description": "Template to use (rust, python, typescript, go)",
					"default":     "rust",
				},
			},
			"required": []string{"name"},
		},
	}
}

func (t *InitTool) Execute(ctx context.Context, args map[string]interface{}) (string, error) {
	name, ok := args["name"].(string)
	if !ok {
		return "", fmt.Errorf("name parameter is required")
	}
	
	template := "rust"
	if t, ok := args["template"].(string); ok {
		template = t
	}
	
	cmd := exec.CommandContext(ctx, "ftl", "init", name, "--template", template)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ftl init failed: %w\nOutput: %s", err, output)
	}
	
	return fmt.Sprintf("Successfully initialized FTL project '%s' with %s template\n%s", name, template, output), nil
}

// BuildTool implements the ftl build command
type BuildTool struct{}

func (t *BuildTool) Definition() ToolDefinition {
	return ToolDefinition{
		Name:        "ftl-build",
		Description: "Build the current FTL project",
		Schema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"watch": map[string]interface{}{
					"type":        "boolean",
					"description": "Enable watch mode for continuous builds",
					"default":     false,
				},
			},
		},
	}
}

func (t *BuildTool) Execute(ctx context.Context, args map[string]interface{}) (string, error) {
	cmdArgs := []string{"build"}
	
	if watch, ok := args["watch"].(bool); ok && watch {
		cmdArgs = append(cmdArgs, "--watch")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ftl build failed: %w\nOutput: %s", err, output)
	}
	
	return fmt.Sprintf("Build completed successfully\n%s", output), nil
}

// UpTool implements the ftl up command
type UpTool struct{}

func (t *UpTool) Definition() ToolDefinition {
	return ToolDefinition{
		Name:        "ftl-up",
		Description: "Start the FTL development server",
		Schema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"watch": map[string]interface{}{
					"type":        "boolean",
					"description": "Enable watch mode for hot reloading",
					"default":     true,
				},
			},
		},
	}
}

func (t *UpTool) Execute(ctx context.Context, args map[string]interface{}) (string, error) {
	cmdArgs := []string{"up"}
	
	if watch, ok := args["watch"].(bool); ok && watch {
		cmdArgs = append(cmdArgs, "--watch")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ftl up failed: %w\nOutput: %s", err, output)
	}
	
	return fmt.Sprintf("Development server started\n%s", output), nil
}

// StatusTool implements the ftl status command
type StatusTool struct{}

func (t *StatusTool) Definition() ToolDefinition {
	return ToolDefinition{
		Name:        "ftl-status",
		Description: "Get the current status of FTL applications",
		Schema: map[string]interface{}{
			"type": "object",
		},
	}
}

func (t *StatusTool) Execute(ctx context.Context, args map[string]interface{}) (string, error) {
	cmd := exec.CommandContext(ctx, "ftl", "status")
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ftl status failed: %w\nOutput: %s", err, output)
	}
	
	return string(output), nil
}

// LogsTool implements the ftl logs command
type LogsTool struct{}

func (t *LogsTool) Definition() ToolDefinition {
	return ToolDefinition{
		Name:        "ftl-logs",
		Description: "Get logs from FTL applications",
		Schema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"follow": map[string]interface{}{
					"type":        "boolean",
					"description": "Follow log output",
					"default":     false,
				},
				"lines": map[string]interface{}{
					"type":        "number",
					"description": "Number of lines to show",
					"default":     50,
				},
			},
		},
	}
}

func (t *LogsTool) Execute(ctx context.Context, args map[string]interface{}) (string, error) {
	cmdArgs := []string{"logs"}
	
	if lines, ok := args["lines"].(float64); ok {
		cmdArgs = append(cmdArgs, "--lines", fmt.Sprintf("%.0f", lines))
	}
	
	if follow, ok := args["follow"].(bool); ok && follow {
		cmdArgs = append(cmdArgs, "--follow")
	}
	
	cmd := exec.CommandContext(ctx, "ftl", cmdArgs...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ftl logs failed: %w\nOutput: %s", err, output)
	}
	
	return string(output), nil
}