package cli

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
)

func newTestCmd() *cobra.Command {
	var coverage bool
	var verbose bool

	cmd := &cobra.Command{
		Use:   "test [path]",
		Short: "Run tests for the FTL application",
		Long: `Run tests for the FTL application and its components.

Runs 'go test' on the specified path or current directory.

Examples:
  ftl test             # Run tests in current directory
  ftl test ./...       # Run all tests recursively
  ftl test -c          # Run with coverage
  ftl test -v ./pkg    # Run with verbose output`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			path := "./..."
			if len(args) > 0 {
				path = args[0]
			}

			// Build go test command
			testArgs := []string{"test"}

			if verbose {
				testArgs = append(testArgs, "-v")
			}

			if coverage {
				testArgs = append(testArgs, "-cover")
			}

			testArgs = append(testArgs, path)

			// Check if we're in a Go module
			if _, err := os.Stat("go.mod"); err != nil {
				return fmt.Errorf("no go.mod found in current directory")
			}

			fmt.Printf("Running: go %s\n", strings.Join(testArgs, " "))

			// Execute go test
			testCmd := exec.Command("go", testArgs...)
			testCmd.Stdout = os.Stdout
			testCmd.Stderr = os.Stderr
			testCmd.Dir, _ = filepath.Abs(".")

			return testCmd.Run()
		},
	}

	cmd.Flags().BoolVarP(&coverage, "coverage", "c", false, "Run tests with coverage")
	cmd.Flags().BoolVarP(&verbose, "verbose", "v", false, "Verbose test output")

	return cmd
}
