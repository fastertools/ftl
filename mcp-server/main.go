package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/fastertools/ftl/mcp-server/tools"
)

func main() {
	// Parse command-line flags
	testMode := flag.Bool("test-mode", false, "Enable test tools for e2e testing")
	flag.Parse()
	
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server (test mode: %v)\n", *testMode)
	
	// Initialize MCP server
	serverOptions := &mcp.ServerOptions{
		Instructions: "FTL server - handles ftl up operations in regular and watch modes",
	}
	
	if *testMode {
		serverOptions.Instructions = "FTL server with test tools enabled for e2e testing"
	}
	
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "mcp-server",
		Version: "1.0.0",
	}, serverOptions)

	// Register all tools
	registry := tools.NewRegistry(server)
	if *testMode {
		registry.EnableTestMode()
	}
	registry.RegisterAll()

	// Run with stdio transport
	server.Run(context.Background(), mcp.NewStdioTransport())
}