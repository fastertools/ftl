package cli

import (
	"fmt"
	"io"
	"os"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var (
	// Version information
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"

	// Configuration
	cfgFile string
	verbose bool
	noColor bool

	// Colors
	successColor = color.New(color.FgGreen, color.Bold)
	errorColor   = color.New(color.FgRed, color.Bold)
	infoColor    = color.New(color.FgCyan)
	warnColor    = color.New(color.FgYellow)

	// For testing - allows redirecting output
	colorOutput io.Writer = os.Stdout
)

// rootCmd represents the base command
var rootCmd = &cobra.Command{
	Use:   "ftl",
	Short: "FTL - Faster Tools for AI agents",
	Long: `FTL is a comprehensive toolkit for building, composing, and deploying 
AI tools on WebAssembly. It provides everything you need to create secure,
high-performance MCP servers that can run anywhere.`,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		if noColor {
			color.NoColor = true
		}
	},
	Version: fmt.Sprintf("%s (commit: %s, built: %s)", version, commit, buildDate),
}

// Execute runs the root command
func Execute() error {
	return rootCmd.Execute()
}

// SetVersion sets the version information
func SetVersion(v, c, b string) {
	version = v
	commit = c
	buildDate = b
	rootCmd.Version = fmt.Sprintf("%s (commit: %s, built: %s)", version, commit, buildDate)
}

func init() {
	cobra.OnInitialize(initConfig)

	// Global flags
	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is ./ftl.yaml)")
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	rootCmd.PersistentFlags().BoolVar(&noColor, "no-color", false, "disable colored output")

	// Bind flags to viper
	_ = viper.BindPFlag("verbose", rootCmd.PersistentFlags().Lookup("verbose"))
	_ = viper.BindPFlag("no-color", rootCmd.PersistentFlags().Lookup("no-color"))

	// Add commands
	rootCmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
		newBuildCmd(),
		newTestCmd(),
		newComponentCmd(),
		newDeployCmd(),
		newAuthCmd(),
		newOrgCmd(),
		newUpCmd(),
		newRegistryCmd(),
		newSynthCmd(),
		newListCmd(),
		newStatusCmd(),
		newDeleteCmd(),
		newLogsCmd(),
	)
}

// initConfig reads in config file and ENV variables if set
func initConfig() {
	if cfgFile != "" {
		viper.SetConfigFile(cfgFile)
	} else {
		viper.AddConfigPath(".")
		viper.SetConfigType("yaml")
		viper.SetConfigName("ftl")
	}

	viper.SetEnvPrefix("FTL")
	viper.AutomaticEnv()

	// Read config file if it exists
	if err := viper.ReadInConfig(); err == nil {
		if verbose {
			fmt.Fprintln(os.Stderr, infoColor.Sprint("Using config file:"), viper.ConfigFileUsed())
		}
	}
}

// Helper functions for consistent output

// Success prints a success message
func Success(format string, args ...interface{}) {
	fmt.Println(successColor.Sprintf("✓ "+format, args...))
}

// Error prints an error message
func Error(format string, args ...interface{}) {
	fmt.Fprintln(os.Stderr, errorColor.Sprintf("✗ "+format, args...))
}

// Info prints an info message
func Info(format string, args ...interface{}) {
	fmt.Println(infoColor.Sprintf("ℹ "+format, args...))
}

// Warn prints a warning message
func Warn(format string, args ...interface{}) {
	fmt.Fprintln(os.Stderr, warnColor.Sprintf("⚠ "+format, args...))
}

// Debug prints a debug message if verbose mode is enabled
func Debug(format string, args ...interface{}) {
	if IsVerbose() {
		fmt.Fprintln(os.Stderr, color.New(color.FgMagenta).Sprintf("» "+format, args...))
	}
}

// Fatal prints an error and exits
func Fatal(format string, args ...interface{}) {
	Error(format, args...)
	os.Exit(1)
}

// PrintStep prints a step in a process
func PrintStep(step int, total int, message string) {
	fmt.Printf("[%d/%d] %s\n", step, total, message)
}

// IsVerbose returns true if verbose mode is enabled
func IsVerbose() bool {
	return viper.GetBool("verbose")
}
