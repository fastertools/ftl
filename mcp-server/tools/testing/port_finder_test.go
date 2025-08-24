package testing

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"testing"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/internal/port"
	"github.com/fastertools/ftl/mcp-server/internal/types"
)

func TestPortFinderHandler_Handle(t *testing.T) {
	tests := []struct {
		name          string
		startPort     int
		endPort       int
		blockPorts    []int
		wantAvailable bool
		wantError     string
	}{
		{
			name:          "find available port in range",
			startPort:     9000,
			endPort:       9010,
			wantAvailable: true,
		},
		{
			name:          "invalid port range",
			startPort:     9010,
			endPort:       9000,
			wantAvailable: false,
			wantError:     "Invalid port range: start_port must be less than end_port",
		},
		{
			name:          "all ports blocked",
			startPort:     9100,
			endPort:       9102,
			blockPorts:    []int{9100, 9101, 9102},
			wantAvailable: false,
			wantError:     "No available ports in specified range",
		},
		{
			name:          "default port range",
			startPort:     0, // Will default to 3000
			endPort:       0, // Will default to 9999
			wantAvailable: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Block specified ports
			var listeners []net.Listener
			for _, p := range tt.blockPorts {
				l, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", p))
				if err == nil {
					listeners = append(listeners, l)
					defer l.Close()
				}
			}
			
			// Create handler
			portManager := port.NewManager(3000)
			handler := NewPortFinderHandler(portManager)
			
			// Create request
			params := &mcp.CallToolParamsFor[types.PortFinderInput]{
				Arguments: types.PortFinderInput{
					StartPort: tt.startPort,
					EndPort:   tt.endPort,
				},
			}
			
			// Execute
			result, err := handler.Handle(context.Background(), nil, params)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			
			// Parse response
			if len(result.Content) == 0 {
				t.Fatal("no content in response")
			}
			
			textContent, ok := result.Content[0].(*mcp.TextContent)
			if !ok {
				t.Fatal("expected TextContent")
			}
			
			var response types.PortFinderResponse
			if err := json.Unmarshal([]byte(textContent.Text), &response); err != nil {
				t.Fatalf("failed to parse response: %v", err)
			}
			
			// Verify
			if response.Available != tt.wantAvailable {
				t.Errorf("available = %v, want %v", response.Available, tt.wantAvailable)
			}
			
			if tt.wantError != "" && response.Error != tt.wantError {
				t.Errorf("error = %v, want %v", response.Error, tt.wantError)
			}
			
			if response.Available && response.Port == 0 {
				t.Error("expected non-zero port when available")
			}
			
			// Clean up listeners
			for _, l := range listeners {
				l.Close()
			}
		})
	}
}

func TestPortFinderHandler_ActualPortCheck(t *testing.T) {
	// This test actually verifies a port is available
	portManager := port.NewManager(3000)
	handler := NewPortFinderHandler(portManager)
	
	params := &mcp.CallToolParamsFor[types.PortFinderInput]{
		Arguments: types.PortFinderInput{
			StartPort: 10000,
			EndPort:   10100,
		},
	}
	
	result, err := handler.Handle(context.Background(), nil, params)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	
	textContent := result.Content[0].(*mcp.TextContent)
	var response types.PortFinderResponse
	json.Unmarshal([]byte(textContent.Text), &response)
	
	if !response.Available {
		t.Skip("No available ports in test range")
	}
	
	// Verify the port is actually available
	l, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", response.Port))
	if err != nil {
		t.Errorf("port %d reported as available but cannot bind: %v", response.Port, err)
	} else {
		l.Close()
	}
}