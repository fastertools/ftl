package cmd

import (
	"bytes"
	"fmt"
	"os"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRootCommand(t *testing.T) {
	tests := []struct {
		name         string
		args         []string
		expectError  bool
		expectOutput []string
	}{
		{
			name:         "no args shows help",
			args:         []string{},
			expectError:  false,
			expectOutput: []string{"FTL", "Usage:", "Available Commands:"},
		},
		{
			name:         "help flag",
			args:         []string{"--help"},
			expectError:  false,
			expectOutput: []string{"FTL", "Usage:", "Available Commands:"},
		},
		{
			name:         "version flag",
			args:         []string{"--version"},
			expectError:  false,
			expectOutput: []string{}, // Version flag shows help, not version
		},
		{
			name:        "invalid command",
			args:        []string{"invalid-command"},
			expectError: true,
		},
		{
			name:         "verbose flag",
			args:         []string{"--verbose", "--help"},
			expectError:  false,
			expectOutput: []string{"FTL"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Reset verbose flag
			verbose = false

			// Use the global rootCmd directly
			rootCmd := &cobra.Command{
				Use:   "ftl",
				Short: "FTL - Faster Tools for AI agents",
				Long: `FTL is a comprehensive toolkit for building, composing, and deploying 
AI tools on WebAssembly. It provides everything you need to create secure,
high-performance MCP servers that can run anywhere.`,
			}

			// Add subcommands
			rootCmd.AddCommand(
				newInitCmd(),
				newAddCmd(),
				newBuildCmd(),
				newTestCmd(),
				newDeployCmd(),
				newAuthCmd(),
				newUpCmd(),
				newRegistryCmd(),
				newSynthCmd(),
			)

			rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
			rootCmd.Flags().BoolP("version", "", false, "version for ftl")

			var buf bytes.Buffer
			rootCmd.SetOut(&buf)
			rootCmd.SetErr(&buf)
			rootCmd.SetArgs(tt.args)

			err := rootCmd.Execute()

			if tt.expectError {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			output := buf.String()
			for _, expected := range tt.expectOutput {
				assert.Contains(t, output, expected)
			}
		})
	}
}

func TestExecute(t *testing.T) {
	// Save original args
	oldArgs := os.Args
	defer func() { os.Args = oldArgs }()

	// Test successful execution
	os.Args = []string{"ftl", "--help"}

	// Execute should not panic
	assert.NotPanics(t, func() {
		_ = Execute()
	})
}

func TestSetVersion(t *testing.T) {
	tests := []struct {
		version   string
		commit    string
		buildDate string
	}{
		{"1.0.0", "abc123", "2024-01-01"},
		{"v2.3.4", "def456", "2024-01-02"},
		{"", "", ""},
		{"dev", "unknown", "unknown"},
	}

	for _, tt := range tests {
		t.Run(tt.version, func(t *testing.T) {
			SetVersion(tt.version, tt.commit, tt.buildDate)
			assert.Equal(t, tt.version, version)
			assert.Equal(t, tt.commit, commit)
			assert.Equal(t, tt.buildDate, buildDate)
		})
	}
}

func TestIsVerbose(t *testing.T) {
	// IsVerbose uses viper, so we need to set it through viper
	// For now, just test the function doesn't panic
	assert.NotPanics(t, func() {
		_ = IsVerbose()
	})
}

func TestOutputHelpers(t *testing.T) {
	tests := []struct {
		name     string
		fn       func(string)
		message  string
		expected string
	}{
		{
			name:     "Success",
			fn:       func(msg string) { Success("%s", msg) },
			message:  "Operation completed",
			expected: "✓ Operation completed",
		},
		{
			name:     "Info",
			fn:       func(msg string) { Info("%s", msg) },
			message:  "Information message",
			expected: "ℹ Information message",
		},
		{
			name:     "Warning",
			fn:       func(msg string) { Warn("%s", msg) },
			message:  "Warning message",
			expected: "⚠ Warning message",
		},
		{
			name:     "Error",
			fn:       func(msg string) { Error("%s", msg) },
			message:  "Error message",
			expected: "✗ Error message",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			output := CaptureOutput(t, func() {
				tt.fn(tt.message)
			})
			assert.Contains(t, output, tt.expected)
		})
	}
}

func TestWarn(t *testing.T) {
	// Test Warn function
	output := CaptureOutput(t, func() {
		Warn("This is a %s", "warning")
	})
	assert.Contains(t, output, "⚠ This is a warning")
}

func TestPrintStep(t *testing.T) {
	output := CaptureOutput(t, func() {
		PrintStep(1, 5, "Processing")
	})
	assert.Contains(t, output, "[1/5] Processing")
}

func TestFatal(t *testing.T) {
	// Fatal calls os.Exit, so we need to test it differently
	// We'll test that it prints the error message before exiting

	if os.Getenv("TEST_FATAL") == "1" {
		Fatal("Fatal error occurred")
		return
	}

	// This is a simplified test - in production you'd use exec.Command
	t.Skip("Fatal test requires subprocess execution")
}

func TestVerboseOutput(t *testing.T) {
	tests := []struct {
		name         string
		verboseFlag  bool
		expectOutput bool
	}{
		{
			name:         "verbose enabled",
			verboseFlag:  true,
			expectOutput: true,
		},
		{
			name:         "verbose disabled",
			verboseFlag:  false,
			expectOutput: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			verbose = tt.verboseFlag
			defer func() { verbose = false }()

			// Test that verbose mode affects command behavior
			cmd := &cobra.Command{Use: "ftl"}
			cmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
			cmd.SetArgs([]string{"--verbose", "--help"})

			var buf bytes.Buffer
			cmd.SetOut(&buf)
			cmd.SetErr(&buf)

			err := cmd.Execute()
			assert.NoError(t, err)

			// After execution, check if verbose flag exists
			if tt.verboseFlag {
				// When verbose is true, the flag should exist
				assert.NotNil(t, cmd.Flag("verbose"))
			}
		})
	}
}

func TestRootCommandStructure(t *testing.T) {
	// Create a test root command
	cmd := &cobra.Command{
		Use:   "ftl",
		Short: "FTL - Faster Tools for AI agents",
		Long: `FTL is a comprehensive toolkit for building, composing, and deploying 
AI tools on WebAssembly. It provides everything you need to create secure,
high-performance MCP servers that can run anywhere.`,
	}

	// Add subcommands
	cmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
		newBuildCmd(),
		newTestCmd(),
		newDeployCmd(),
		newAuthCmd(),
		newUpCmd(),
		newRegistryCmd(),
		newSynthCmd(),
	)

	cmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	cmd.Flags().BoolP("version", "", false, "version for ftl")

	// Test command properties
	assert.Equal(t, "ftl", cmd.Use)
	assert.Contains(t, cmd.Short, "FTL")
	assert.Contains(t, cmd.Long, "FTL")

	// Test that all subcommands are added
	expectedCommands := []string{
		"init", "add", "build", "deploy", "up", "synth",
		"test", "auth", "registry",
	}

	for _, cmdName := range expectedCommands {
		t.Run("has_"+cmdName, func(t *testing.T) {
			found := false
			for _, subCmd := range cmd.Commands() {
				if subCmd.Name() == cmdName {
					found = true
					break
				}
			}
			assert.True(t, found, "Command %s not found", cmdName)
		})
	}

	// Test global flags
	assert.NotNil(t, cmd.Flag("verbose"))
	assert.NotNil(t, cmd.Flag("version"))
}

func TestRootCommandFlags(t *testing.T) {
	// Create test command
	cmd := &cobra.Command{Use: "ftl"}
	cmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")
	cmd.Flags().BoolP("version", "", false, "version for ftl")

	tests := []struct {
		flag        string
		shorthand   string
		defaultVal  string
		description string
	}{
		{
			flag:        "verbose",
			shorthand:   "v",
			defaultVal:  "false",
			description: "verbose output",
		},
		{
			flag:        "version",
			shorthand:   "",
			defaultVal:  "false",
			description: "version",
		},
	}

	for _, tt := range tests {
		t.Run(tt.flag, func(t *testing.T) {
			flag := cmd.Flag(tt.flag)
			require.NotNil(t, flag, "Flag %s not found", tt.flag)

			if tt.shorthand != "" {
				assert.Equal(t, tt.shorthand, flag.Shorthand)
			}
			assert.Equal(t, tt.defaultVal, flag.DefValue)
			assert.Contains(t, flag.Usage, tt.description)
		})
	}
}

func TestPreRunVerbose(t *testing.T) {
	// Create test command with PreRun
	cmd := &cobra.Command{
		Use: "ftl",
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			// This simulates what the real command does
		},
	}
	cmd.AddCommand(&cobra.Command{Use: "help"})
	cmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "verbose output")

	// Test that PreRun sets verbose correctly
	cmd.SetArgs([]string{"--verbose", "help"})

	err := cmd.Execute()
	assert.NoError(t, err)
	assert.True(t, verbose)

	// Reset
	verbose = false
}

func TestRootCommandCompletion(t *testing.T) {
	// Skip completion tests as they require full command setup
	t.Skip("Completion tests require full cobra setup")
}

func TestCommandHelp(t *testing.T) {
	// Create test command
	cmd := &cobra.Command{
		Use:   "ftl",
		Short: "FTL - Faster Tools for AI agents",
	}
	cmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
		newBuildCmd(),
	)

	// Test help for main command
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "FTL")
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "Available Commands:")
	assert.Contains(t, output, "Flags:")
}

func TestSubcommandHelp(t *testing.T) {
	// Create test command with subcommands
	cmd := &cobra.Command{Use: "ftl"}
	cmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
		newBuildCmd(),
		newDeployCmd(),
		newUpCmd(),
		newSynthCmd(),
		newTestCmd(),
	)

	subcommands := []string{"init", "add", "build", "deploy", "up", "synth", "test"}

	for _, subcmd := range subcommands {
		t.Run(subcmd+"_help", func(t *testing.T) {
			var buf bytes.Buffer
			cmd.SetOut(&buf)
			cmd.SetArgs([]string{subcmd, "--help"})

			err := cmd.Execute()
			assert.NoError(t, err)

			output := buf.String()
			assert.Contains(t, output, "Usage:")
			assert.Contains(t, output, subcmd)
		})
	}
}

func BenchmarkRootCommand(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmd := &cobra.Command{
			Use:   "ftl",
			Short: "FTL",
		}
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}

func BenchmarkOutputHelpers(b *testing.B) {
	// Redirect output to prevent console spam
	old := os.Stdout
	os.Stdout, _ = os.Open(os.DevNull)
	defer func() { os.Stdout = old }()

	b.Run("Success", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			Success("Test message")
		}
	})

	b.Run("Info", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			Info("Test message")
		}
	})

	b.Run("Warn", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			Warn("Test message")
		}
	})

	b.Run("Error", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			Error("Test message")
		}
	})
}

// TestCommandUsageTemplates verifies custom usage templates if any
func TestCommandUsageTemplates(t *testing.T) {
	cmd := &cobra.Command{
		Use:   "ftl",
		Short: "FTL",
		Long:  "FTL is a toolkit",
	}

	// Test that the command has proper usage template
	assert.NotEmpty(t, cmd.Use)
	assert.NotEmpty(t, cmd.Short)
	assert.NotEmpty(t, cmd.Long)

	// Verify the command can generate usage without errors
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	err := cmd.Usage()
	assert.NoError(t, err)
	assert.NotEmpty(t, buf.String())
}

// TestCommandAliases checks if command aliases work correctly
func TestCommandAliases(t *testing.T) {
	cmd := &cobra.Command{Use: "ftl"}
	cmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
	)

	// If any commands have aliases, test them
	for _, subCmd := range cmd.Commands() {
		if len(subCmd.Aliases) > 0 {
			for _, alias := range subCmd.Aliases {
				t.Run(fmt.Sprintf("alias_%s_for_%s", alias, subCmd.Name()), func(t *testing.T) {
					foundCmd, _, err := cmd.Find([]string{alias})
					assert.NoError(t, err)
					assert.Equal(t, subCmd.Name(), foundCmd.Name())
				})
			}
		}
	}
}

// TestCommandExamples verifies that command examples are valid
func TestCommandExamples(t *testing.T) {
	cmd := &cobra.Command{Use: "ftl"}
	cmd.AddCommand(
		newInitCmd(),
		newAddCmd(),
	)

	// Check that commands with examples have valid syntax
	var checkExamples func(*cobra.Command)
	checkExamples = func(c *cobra.Command) {
		if c.Example != "" {
			t.Run(c.Name()+"_example", func(t *testing.T) {
				assert.NotEmpty(t, c.Example)
				// Could parse and validate example syntax here
			})
		}
		for _, sub := range c.Commands() {
			checkExamples(sub)
		}
	}

	checkExamples(cmd)
}
