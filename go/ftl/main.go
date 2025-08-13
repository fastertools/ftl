package main

import (
	"os"

	"github.com/fastertools/ftl-cli/go/ftl/cmd"
)

// Version information set at build time
var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
)

func main() {
	// Set version information for commands to use
	cmd.SetVersion(version, commit, buildDate)

	// Execute the root command
	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}