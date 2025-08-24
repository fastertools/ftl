package testing

import (
	"context"
	"encoding/json"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/port"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

// PortFinderHandler handles port discovery requests
type PortFinderHandler struct {
	portManager *port.Manager
}

// NewPortFinderHandler creates a new port finder handler
func NewPortFinderHandler(portManager *port.Manager) *PortFinderHandler {
	return &PortFinderHandler{
		portManager: portManager,
	}
}

// Handle processes the port finder request
func (h *PortFinderHandler) Handle(ctx context.Context, ss *mcp.ServerSession, params *mcp.CallToolParamsFor[types.PortFinderInput]) (*mcp.CallToolResultFor[struct{}], error) {
	startPort := params.Arguments.StartPort
	endPort := params.Arguments.EndPort
	
	// Set defaults if not provided
	if startPort == 0 {
		startPort = 3000
	}
	if endPort == 0 {
		endPort = 9999
	}
	
	// Validate port range
	if startPort > endPort {
		response := types.PortFinderResponse{
			Port:      0,
			Available: false,
			Error:     "Invalid port range: start_port must be less than end_port",
		}
		responseJSON, _ := json.Marshal(response)
		return &mcp.CallToolResultFor[struct{}]{
			Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
		}, nil
	}
	
	// Find available port in range
	for p := startPort; p <= endPort; p++ {
		if port.IsAvailable(p) {
			response := types.PortFinderResponse{
				Port:      p,
				Available: true,
			}
			responseJSON, _ := json.Marshal(response)
			return &mcp.CallToolResultFor[struct{}]{
				Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
			}, nil
		}
	}
	
	// No available port found
	response := types.PortFinderResponse{
		Port:      0,
		Available: false,
		Error:     "No available ports in specified range",
	}
	responseJSON, _ := json.Marshal(response)
	return &mcp.CallToolResultFor[struct{}]{
		Content: []mcp.Content{&mcp.TextContent{Text: string(responseJSON)}},
	}, nil
}