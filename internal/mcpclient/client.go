package mcpclient

import (
	"context"
	"fmt"
	"os/exec"
	"sync"
	"time"
	
	"github.com/modelcontextprotocol/go-sdk/mcp"
)

// Tool is a type alias for the MCP SDK Tool type to maintain backward compatibility
type Tool = mcp.Tool

// Client handles communication with the FTL MCP server using the official Go SDK
type Client struct {
	serverPath string
	serverArgs []string
	impl       *mcp.Implementation
	client     *mcp.Client
	session    *mcp.ClientSession
	transport  mcp.Transport
	mu         sync.Mutex
}

// NewClient creates a new MCP client instance
func NewClient(serverPath string) *Client {
	return &Client{
		serverPath: serverPath,
		impl: &mcp.Implementation{
			Name:    "ftl-web-client", 
			Version: "1.0.0",
		},
	}
}

// NewClientWithArgs creates a new MCP client instance with command arguments
func NewClientWithArgs(serverPath string, args []string) *Client {
	return &Client{
		serverPath: serverPath,
		serverArgs: args,
		impl: &mcp.Implementation{
			Name:    "ftl-web-client",
			Version: "1.0.0",
		},
	}
}

// Connect establishes connection to the MCP server
func (c *Client) Connect(ctx context.Context) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	if c.session != nil {
		return nil // Already connected
	}
	
	// Create command transport
	var cmd *exec.Cmd
	if len(c.serverArgs) > 0 {
		cmd = exec.Command(c.serverPath, c.serverArgs...)
	} else {
		cmd = exec.Command(c.serverPath)
	}
	
	transport := mcp.NewCommandTransport(cmd)
	
	// Create client and connect
	client := mcp.NewClient(c.impl, nil)
	session, err := client.Connect(ctx, transport)
	if err != nil {
		return fmt.Errorf("failed to connect to MCP server: %w", err)
	}
	
	c.client = client
	c.session = session  
	c.transport = transport
	return nil
}

// EnsureConnection ensures we have a connection (backward compatibility)
func (c *Client) EnsureConnection() error {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return c.Connect(ctx)
}

// CallTool calls an MCP tool with the given arguments
func (c *Client) CallTool(toolName string, args map[string]interface{}) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	
	return c.CallToolWithContext(ctx, toolName, args)
}

// CallToolWithContext calls an MCP tool with context for timeout control
func (c *Client) CallToolWithContext(ctx context.Context, toolName string, args map[string]interface{}) (string, error) {
	if err := c.Connect(ctx); err != nil {
		return "", err
	}
	
	c.mu.Lock()
	defer c.mu.Unlock()
	
	result, err := c.session.CallTool(ctx, &mcp.CallToolParams{
		Name:      toolName,
		Arguments: args,
	})
	if err != nil {
		return "", fmt.Errorf("tool call failed: %w", err)
	}
	
	// Extract text content from result
	if len(result.Content) > 0 {
		if textContent, ok := result.Content[0].(*mcp.TextContent); ok {
			return textContent.Text, nil
		}
	}
	
	return "No response from MCP server", nil
}

// ListTools returns available MCP tools
func (c *Client) ListTools(ctx context.Context) ([]*mcp.Tool, error) {
	if err := c.Connect(ctx); err != nil {
		return nil, err
	}
	
	c.mu.Lock()
	defer c.mu.Unlock()
	
	var tools []*mcp.Tool
	for tool, err := range c.session.Tools(ctx, nil) {
		if err != nil {
			return nil, fmt.Errorf("failed to list tools: %w", err)
		}
		tools = append(tools, tool)
	}
	return tools, nil
}

// Close closes the MCP client connection
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	
	if c.session != nil {
		err := c.session.Close()
		c.session = nil
		c.client = nil
		c.transport = nil
		return err
	}
	return nil
}

// Cleanup cleans up the MCP client resources (backward compatibility)
func (c *Client) Cleanup() {
	c.Close()
}