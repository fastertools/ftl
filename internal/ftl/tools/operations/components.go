package operations

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/ftl/ftl"
	"github.com/fastertools/ftl/internal/ftl/types"
)

// ComponentsHandler handles ftl component list operations
type ComponentsHandler struct {
	ftlCommander *ftl.Commander
}

// NewComponentsHandler creates a new components handler
func NewComponentsHandler(ftlCommander *ftl.Commander) *ComponentsHandler {
	return &ComponentsHandler{
		ftlCommander: ftlCommander,
	}
}

// Handle processes the ftl component list request
func (h *ComponentsHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.ListComponentsInput]) (*mcp.CallToolResultFor[struct{}], error) {
	fmt.Fprintf(os.Stderr, "DEBUG: listComponents function called\n")
	projectPath := params.Arguments.ProjectPath

	// Log the incoming payload to stderr
	fmt.Fprintf(os.Stderr, "DEBUG: list_components received payload: project_path='%s'\n", projectPath)

	// Validate project directory
	if err := ftl.ValidateProjectPath(projectPath); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		response := types.ListComponentsResponse{
			Success:     false,
			Components:  []types.Component{},
			Count:       0,
			ProjectPath: projectPath,
			Error:       err.Error(),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Execute component list command
	output, err := h.ftlCommander.ExecuteComponentList(projectPath)
	
	if err != nil {
		response := types.ListComponentsResponse{
			Success:     false,
			Components:  []types.Component{},
			Count:       0,
			ProjectPath: projectPath,
			Error:       fmt.Sprintf("Component list failed: %s", err),
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}

	// Parse the component list output
	components := parseComponentOutput(output)

	response := types.ListComponentsResponse{
		Success:     true,
		Components:  components,
		Count:       len(components),
		ProjectPath: projectPath,
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}

// parseComponentOutput parses the output from ftl component list
func parseComponentOutput(output string) []types.Component {
	var components []types.Component
	
	// Split output into lines and process each line
	lines := strings.Split(strings.TrimSpace(output), "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		
		// Parse component information
		// Expected format might vary, but let's handle common patterns
		component := parseComponentLine(line)
		if component.Name != "" {
			components = append(components, component)
		}
	}
	
	return components
}

// parseComponentLine parses a single line of component output
func parseComponentLine(line string) types.Component {
	// Handle different possible output formats from ftl component list
	
	// If line looks like "componentName (language) - description"
	if strings.Contains(line, "(") && strings.Contains(line, ")") {
		parts := strings.SplitN(line, "(", 2)
		if len(parts) == 2 {
			name := strings.TrimSpace(parts[0])
			rest := parts[1]
			
			// Extract language
			langEnd := strings.Index(rest, ")")
			if langEnd > 0 {
				language := strings.TrimSpace(rest[:langEnd])
				description := ""
				if len(rest) > langEnd+1 {
					description = strings.TrimSpace(rest[langEnd+1:])
					if strings.HasPrefix(description, "-") {
						description = strings.TrimSpace(description[1:])
					}
				}
				
				return types.Component{
					Name:        name,
					Language:    language,
					Directory:   name, // Default to name, could be enhanced
					Description: description,
				}
			}
		}
	}
	
	// Simple fallback - treat entire line as component name
	if line != "" && !strings.HasPrefix(line, "Error") && !strings.HasPrefix(line, "No components") {
		return types.Component{
			Name:      line,
			Language:  "unknown",
			Directory: line,
		}
	}
	
	return types.Component{}
}