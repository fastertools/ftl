package cli

import (
	"context"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/types"
	"github.com/fastertools/ftl/mcp-server/tools"
)

// newDevMcpCmd creates the dev mcp command
func newDevMcpCmd() *cobra.Command {
	var (
		ftlPath  string
		testMode bool
	)

	cmd := &cobra.Command{
		Use:   "mcp",
		Short: "Run MCP server for AI tool integration",
		Long: `Start an MCP (Model Context Protocol) server that exposes FTL functionality
to AI tools like Claude Desktop. The server runs over stdio and provides
tools for project management, building, deployment, and monitoring.`,
		Example: `  # Run MCP server (for Claude Desktop)
  ftl dev mcp

  # Test MCP server with debugging output  
  ftl dev mcp --verbose
  
  # Use specific FTL binary path
  ftl dev mcp --ftl-path /path/to/ftl`,
		RunE: func(cmd *cobra.Command, args []string) error {
			return runDevMcp(cmd, args, ftlPath, testMode)
		},
	}

	// Add flags
	cmd.Flags().StringVar(&ftlPath, "ftl-path", "", "Path to FTL binary to use (defaults to 'ftl')")
	cmd.Flags().BoolVar(&testMode, "test-mode", false, "Enable test tools for e2e testing")

	return cmd
}

func runDevMcp(cmd *cobra.Command, args []string, ftlPath string, testMode bool) error {
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server (test mode: %v, ftl path: %s)\n", testMode, ftlPath)
	
	// Create tool configuration
	toolConfig := &types.ToolConfig{}
	if ftlPath != "" {
		toolConfig.FTLPath = ftlPath
	}
	
	// Initialize MCP server
	serverOptions := &mcp.ServerOptions{
		Instructions: "FTL server - handles ftl operations with configurable binary path",
	}
	
	if testMode {
		serverOptions.Instructions = "FTL server with test tools enabled for e2e testing"
	}
	
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "mcp-server",
		Version: "1.0.0",
	}, serverOptions)

	// Register all tools with configuration
	registry := tools.NewRegistryWithConfig(server, toolConfig)
	if testMode {
		registry.EnableTestMode()
	}
	registry.RegisterAll()

	// Run with stdio transport
	return server.Run(context.Background(), mcp.NewStdioTransport())
}