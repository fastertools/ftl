package main

import (
	"os"

	"github.com/fastertools/ftl-cli/internal/cli"
)

// Version information set at build time
var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
)

func main() {
	// Set version information for commands to use
	cli.SetVersion(version, commit, buildDate)

	// Execute the root command
	if err := cli.Execute(); err != nil {
		os.Exit(1)
	}
}
