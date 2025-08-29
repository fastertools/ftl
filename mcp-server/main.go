package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/internal/types"
	"github.com/fastertools/ftl/mcp-server/tools"
)

func main() {
	// Parse command-line flags
	testMode := flag.Bool("test-mode", false, "Enable test tools for e2e testing")
	ftlPath := flag.String("ftl-path", "", "Path to FTL binary to use")
	flag.Parse()
	
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server (test mode: %v, ftl path: %s)\n", *testMode, *ftlPath)
	
	// Create tool configuration
	toolConfig := &types.ToolConfig{}
	if *ftlPath != "" {
		toolConfig.FTLPath = *ftlPath
	}
	
	// Initialize MCP server
	serverOptions := &mcp.ServerOptions{
		Instructions: "FTL server - handles ftl operations with configurable binary path",
	}
	
	if *testMode {
		serverOptions.Instructions = "FTL server with test tools enabled for e2e testing"
	}
	
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "mcp-server",
		Version: "1.0.0",
	}, serverOptions)

	// Register all tools with configuration
	registry := tools.NewRegistryWithConfig(server, toolConfig)
	if *testMode {
		registry.EnableTestMode()
	}
	registry.RegisterAll()

	// Run with stdio transport
	server.Run(context.Background(), mcp.NewStdioTransport())
}