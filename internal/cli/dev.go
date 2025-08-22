package cli

import (
	"github.com/spf13/cobra"
)

// newDevCmd creates the dev command group
func newDevCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dev",
		Short: "Development tools and utilities",
		Long: `Development tools including web console for real-time monitoring 
and MCP server for AI tool integration.`,
		Example: `  # Launch web development console
  ftl dev console

  # Start MCP server for Claude Desktop integration  
  ftl dev mcp`,
	}

	// Add subcommands
	cmd.AddCommand(
		newDevConsoleCmd(),
		newDevMcpCmd(),
	)

	return cmd
}