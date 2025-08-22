package main

import (
	"context"
	"fmt"
	"os"

	"github.com/modelcontextprotocol/go-sdk/mcp"
	"github.com/modelcontextprotocol/mcp-server/tools"
)

func main() {
	fmt.Fprintf(os.Stderr, "DEBUG: Starting FTL MCP server\n")
	
	// Initialize MCP server
	server := mcp.NewServer(&mcp.Implementation{
		Name:    "mcp-server",
		Version: "1.0.0",
	}, &mcp.ServerOptions{
		Instructions: "FTL server - handles ftl up operations in regular and watch modes",
	})

	// Register all tools
	registry := tools.NewRegistry(server)
	registry.RegisterAll()

	// Run with stdio transport
	server.Run(context.Background(), mcp.NewStdioTransport())
}