package cli

import (
	"context"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/modelcontextprotocol/mcp-server/tools"
)

// newDevMcpCmd creates the dev mcp command
func newDevMcpCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "mcp",
		Short: "Run MCP server for AI tool integration",
		Long: `Start an MCP (Model Context Protocol) server that exposes FTL functionality
to AI tools like Claude Desktop. The server runs over stdio and provides
tools for project management, building, deployment, and monitoring.`,
		Example: `  # Run MCP server (for Claude Desktop)
  ftl dev mcp

  # Test MCP server with debugging output  
  ftl dev mcp --verbose`,
		RunE: runDevMcp,
	}

	return cmd
}

func runDevMcp(cmd *cobra.Command, args []string) error {
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server\n")
	
	// Initialize MCP server exactly like the original
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "mcp-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "FTL server - handles ftl up operations in regular and watch modes",
	})

	// Register all tools using the tools registry
	registry := tools.NewRegistry(server)
	registry.RegisterAll()

	// Run with stdio transport
	return server.Run(context.Background(), mcp.NewStdioTransport())
}