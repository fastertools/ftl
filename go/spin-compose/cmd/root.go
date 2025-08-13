package cmd

import (
	"os"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
)

var (
	// Build-time variables (set by linker)
	version   = "1.0.0"
	commit    = "unknown"
	buildTime = "unknown"
	
	// Color functions for consistent styling
	successColor = color.New(color.FgGreen, color.Bold)
	errorColor   = color.New(color.FgRed, color.Bold)
	warningColor = color.New(color.FgYellow, color.Bold)
	infoColor    = color.New(color.FgCyan, color.Bold)
	dimColor     = color.New(color.Faint)
)

var rootCmd = &cobra.Command{
	Use:   "spin-compose",
	Short: "Infrastructure as Code for WebAssembly",
	Long: `spin-compose is a modern Infrastructure as Code tool for WebAssembly applications.

It allows you to define, synthesize, and manage Spin applications using high-level
constructs and CUE-based configuration synthesis.`,
	Version: version,
	PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
		// Check if we're running a command that requires the synthesis engine
		if cmd.Name() == "synth" || cmd.Name() == "validate" || cmd.Name() == "diff" {
			// The synthesis engine is built-in, no external dependencies required
		}
		return nil
	},
}

// Execute runs the root command
func Execute() error {
	return rootCmd.Execute()
}

func init() {
	// Set custom help template for beautiful output
	rootCmd.SetHelpTemplate(helpTemplate)
	
	// Add subcommands
	rootCmd.AddCommand(initCmd)
	rootCmd.AddCommand(synthCmd)
	rootCmd.AddCommand(validateCmd)
	rootCmd.AddCommand(diffCmd)
	rootCmd.AddCommand(constructCmd)
}

// printSuccess prints a success message with green styling
func printSuccess(format string, args ...interface{}) {
	successColor.Printf("✓ "+format+"\n", args...)
}

// printError prints an error message with red styling
func printError(format string, args ...interface{}) {
	errorColor.Fprintf(os.Stderr, "✗ "+format+"\n", args...)
}

// printWarning prints a warning message with yellow styling
func printWarning(format string, args ...interface{}) {
	warningColor.Printf("⚠ "+format+"\n", args...)
}

// printInfo prints an info message with cyan styling
func printInfo(format string, args ...interface{}) {
	infoColor.Printf("ℹ "+format+"\n", args...)
}

// printDim prints a dimmed message
func printDim(format string, args ...interface{}) {
	dimColor.Printf(format+"\n", args...)
}

var helpTemplate = `{{.Long}}

Usage:
  {{.UseLine}}{{if .HasAvailableSubCommands}}

Available Commands:{{range .Commands}}{{if (or .IsAvailableCommand (eq .Name "help"))}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}{{if .HasAvailableLocalFlags}}

Flags:
{{.LocalFlags.FlagUsages | trimTrailingWhitespaces}}{{end}}{{if .HasAvailableInheritedFlags}}

Global Flags:
{{.InheritedFlags.FlagUsages | trimTrailingWhitespaces}}{{end}}{{if .HasHelpSubCommands}}

Additional help topics:{{range .Commands}}{{if .IsAdditionalHelpTopicCommand}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}

Use "{{.CommandPath}} [command] --help" for more information about a command.
`