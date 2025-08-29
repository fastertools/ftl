package cli

import (
	"context"
	"fmt"
	"net/http"
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
		httpAddr string
		verbose  bool
	)

	cmd := &cobra.Command{
		Use:   "mcp",
		Short: "Run MCP server for AI tool integration",
		Long: `Start an MCP (Model Context Protocol) server that exposes FTL functionality
to AI tools. Supports both stdio (for desktop integration) and HTTP (for web integration).`,
		Example: `  # Run MCP server over stdio (for Claude Desktop)
  ftl dev mcp

  # Run MCP server over HTTP (for web clients)
  ftl dev mcp --http :8080
  
  # Run with test mode enabled
  ftl dev mcp --test-mode --verbose`,
		RunE: func(cmd *cobra.Command, args []string) error {
			return runDevMcp(cmd, args, ftlPath, testMode, httpAddr, verbose)
		},
	}

	// Add flags
	cmd.Flags().StringVar(&ftlPath, "ftl-path", "", "Path to FTL binary to use (defaults to 'ftl')")
	cmd.Flags().BoolVar(&testMode, "test-mode", false, "Enable test tools for e2e testing")
	cmd.Flags().StringVar(&httpAddr, "http", "", "HTTP address (e.g., :8080) - uses stdio if not set")
	cmd.Flags().BoolVar(&verbose, "verbose", false, "Enable verbose logging")

	return cmd
}

func runDevMcp(cmd *cobra.Command, args []string, ftlPath string, testMode bool, httpAddr string, verbose bool) error {
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server (test mode: %v, ftl path: %s, http: %s)\n", testMode, ftlPath, httpAddr)
	
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

	// Choose transport based on flags
	if httpAddr != "" {
		return runHTTPServer(server, httpAddr)
	} else {
		return runStdioServer(server, verbose)
	}
}

// runHTTPServer runs the MCP server with HTTP transport
func runHTTPServer(server *mcp.Server, addr string) error {
	handler := mcp.NewStreamableHTTPHandler(func(*http.Request) *mcp.Server {
		return server
	}, nil)
	
	fmt.Fprintf(os.Stderr, "FTL MCP server listening at %s\n", addr)
	return http.ListenAndServe(addr, handler)
}

// runStdioServer runs the MCP server with stdio transport
func runStdioServer(server *mcp.Server, verbose bool) error {
	var transport mcp.Transport = mcp.NewStdioTransport()
	if verbose {
		transport = mcp.NewLoggingTransport(transport, os.Stderr)
	}
	
	return server.Run(context.Background(), transport)
}