package cli

import (
	"fmt"
	"os"
	"testing"
)

// TestHelperProcess is not a real test. It's used to mock exec.Command
func TestHelperProcess(t *testing.T) {
	if os.Getenv("GO_WANT_HELPER_PROCESS") != "1" {
		return
	}

	args := os.Args
	for i, arg := range args {
		if arg == "--" {
			args = args[i+1:]
			break
		}
	}

	if len(args) == 0 {
		fmt.Fprintf(os.Stderr, "No command specified")
		os.Exit(1)
	}

	cmd := args[0]
	cmdArgs := args[1:]

	// Mock different commands
	switch cmd {
	case "spin":
		handleSpinCommand(cmdArgs)
	case "docker":
		handleDockerCommand(cmdArgs)
	case "make":
		handleMakeCommand(cmdArgs)
	case "go":
		handleGoCommand(cmdArgs)
	case "ftl":
		handleFTLCommand(cmdArgs)
	default:
		fmt.Fprintf(os.Stderr, "Unknown command: %s", cmd)
		os.Exit(1)
	}

	os.Exit(0)
}
