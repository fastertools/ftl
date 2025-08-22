package cli

import (
	"fmt"
	"net/http"
	"os"
	
	"github.com/fastertools/ftl/internal/handlers"
	"github.com/fastertools/ftl/internal/mcpclient"
	"github.com/fastertools/ftl/internal/network"
	"github.com/spf13/cobra"
)

// newDevConsoleCmd creates the dev console command
func newDevConsoleCmd() *cobra.Command {
	var port int

	cmd := &cobra.Command{
		Use:   "console",
		Short: "Launch web development console",
		Long: `Start a web-based development console that provides real-time monitoring
of build processes, logs, component status, and project management through
an HTMX-powered interface.`,
		Example: `  # Launch console on default port
  ftl dev console

  # Launch console on specific port
  ftl dev console --port 8080`,
		RunE: func(cmd *cobra.Command, args []string) error {
			return runDevConsole(port)
		},
	}

	// Add flags
	cmd.Flags().IntVarP(&port, "port", "p", 8080, "port to run console on")

	return cmd
}

func runDevConsole(port int) error {
	Info("Starting FTL development console...")
	
	// Path to our FTL MCP server - use self as subprocess
	mcpServerPath := os.Args[0] // Use self as the MCP server
	mcpClient := mcpclient.NewClientWithArgs(mcpServerPath, []string{"dev", "mcp"})

	// Cleanup on exit
	defer mcpClient.Cleanup()

	// Create handler with dependencies
	handler := handlers.NewHandler(mcpClient)

	// Setup routes
	http.HandleFunc("/", handler.HandleIndex)
	http.HandleFunc("/mcp", handler.HandleMCP)

	// HTMX-specific endpoints that return HTML fragments
	http.HandleFunc("/htmx/ftl/start", handler.HandleFTLStart)
	http.HandleFunc("/htmx/ftl/stop", handler.HandleFTLStop)
	http.HandleFunc("/htmx/ftl/watch/start", handler.HandleWatchStart)
	http.HandleFunc("/htmx/ftl/watch/stop", handler.HandleWatchStop)
	http.HandleFunc("/htmx/logs/poll", handler.HandleLogsPoll)
	http.HandleFunc("/htmx/status/poll", handler.HandleStatusPoll)
	http.HandleFunc("/htmx/process/stop", handler.HandleProcessStop)

	// Project management endpoints
	http.HandleFunc("/htmx/project/add-form", handler.HandleProjectAddForm)
	http.HandleFunc("/htmx/project/cancel-form", handler.HandleProjectCancelForm)
	http.HandleFunc("/htmx/project/add", handler.HandleProjectAdd)
	http.HandleFunc("/htmx/project/remove", handler.HandleProjectRemove)
	http.HandleFunc("/htmx/project/switch", handler.HandleProjectSwitch)
	http.HandleFunc("/htmx/project/reload", handler.HandleProjectReload)
	
	// Tools endpoints
	http.HandleFunc("/tools-list", handler.HandleToolsList)
	http.HandleFunc("/tool-params/", handler.HandleToolParams)

	Success("Starting server on port %d...", port)

	// Try to start server on the specified port
	if err := network.StartServerOnAvailablePort(port, nil); err != nil {
		return fmt.Errorf("failed to start server: %w", err)
	}
	
	return nil
}