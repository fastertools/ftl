//go:build !production
// +build !production

package cli

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// ExecCommand is a variable that can be mocked in tests
var ExecCommand = exec.Command

// MockExecCommand represents a mocked external command
type MockExecCommand struct {
	Command      string
	Args         []string
	ReturnOutput string
	ReturnError  error
	ReturnCode   int
}

// MockCommandExecutor manages mocked commands
type MockCommandExecutor struct {
	Commands map[string]*MockExecCommand
	History  []string
}

// NewMockCommandExecutor creates a new mock executor
func NewMockCommandExecutor() *MockCommandExecutor {
	return &MockCommandExecutor{
		Commands: make(map[string]*MockExecCommand),
		History:  []string{},
	}
}

// RegisterCommand registers a mock command
func (m *MockCommandExecutor) RegisterCommand(cmd string, mock *MockExecCommand) {
	m.Commands[cmd] = mock
}

// Execute simulates command execution
func (m *MockCommandExecutor) Execute(name string, args ...string) ([]byte, error) {
	cmdKey := name + " " + strings.Join(args, " ")
	m.History = append(m.History, cmdKey)

	if mock, ok := m.Commands[name]; ok {
		if mock.ReturnError != nil {
			return nil, mock.ReturnError
		}
		return []byte(mock.ReturnOutput), nil
	}

	// Default behavior for unmocked commands
	return []byte(""), nil
}

// GetHistory returns command execution history
func (m *MockCommandExecutor) GetHistory() []string {
	return m.History
}

// ResetHistory clears the command history
func (m *MockCommandExecutor) ResetHistory() {
	m.History = []string{}
}

// MockExecCommandHelper is used to create fake commands for testing
func MockExecCommandHelper(command string, args ...string) *exec.Cmd {
	cs := []string{"-test.run=TestHelperProcess", "--", command}
	cs = append(cs, args...)
	cmd := exec.Command(os.Args[0], cs...) // #nosec G204 - test mock code only
	cmd.Env = []string{"GO_WANT_HELPER_PROCESS=1"}
	return cmd
}

func handleSpinCommand(args []string) {
	if len(args) == 0 {
		fmt.Println("spin version 2.0.0")
		return
	}

	switch args[0] {
	case "build":
		fmt.Println("Building application...")
		fmt.Println("✓ Built successfully")
	case "up":
		fmt.Println("Starting application...")
		fmt.Println("Available on http://localhost:3000")
	case "deploy":
		fmt.Println("Deploying application...")
		fmt.Println("✓ Deployed successfully")
	case "registry":
		if len(args) > 1 && args[1] == "push" {
			fmt.Println("Pushing to registry...")
			fmt.Println("✓ Pushed successfully")
		}
	default:
		fmt.Printf("Unknown spin command: %s\n", args[0])
		os.Exit(1)
	}
}

func handleDockerCommand(args []string) {
	if len(args) == 0 {
		fmt.Println("Docker version 24.0.0")
		return
	}

	switch args[0] {
	case "build":
		fmt.Println("Building Docker image...")
		fmt.Println("✓ Image built successfully")
	case "run":
		fmt.Println("Running container...")
	case "push":
		fmt.Println("Pushing image...")
		fmt.Println("✓ Pushed successfully")
	default:
		fmt.Printf("Unknown docker command: %s\n", args[0])
	}
}

func handleMakeCommand(args []string) {
	if len(args) == 0 {
		args = []string{"all"}
	}

	target := args[0]
	fmt.Printf("make: Entering directory\n")
	fmt.Printf("Building target '%s'...\n", target)
	fmt.Printf("✓ Build complete\n")
}

func handleGoCommand(args []string) {
	if len(args) == 0 {
		fmt.Println("go version go1.21.0")
		return
	}

	switch args[0] {
	case "run":
		// Simulate Go CDK synthesis
		fmt.Println(`spin_manifest_version = 2
[application]
name = "test-app"
version = "0.1.0"`)
	case "build":
		fmt.Println("Building Go application...")
	case "test":
		fmt.Println("PASS")
		fmt.Println("ok  	test-package	0.001s")
	case "mod":
		if len(args) > 1 && args[1] == "tidy" {
			fmt.Println("go: downloading modules...")
		}
	default:
		fmt.Printf("Unknown go command: %s\n", args[0])
	}
}

func handleFTLCommand(args []string) {
	if len(args) == 0 {
		fmt.Println("ftl version 0.6.0")
		return
	}

	switch args[0] {
	case "synth":
		// Look for -o flag to determine output file
		outputFile := ""
		for i := 0; i < len(args); i++ {
			if args[i] == "-o" && i+1 < len(args) {
				outputFile = args[i+1]
				break
			}
		}

		// Generate a mock spin.toml content
		spinTOML := `spin_manifest_version = 2

[application]
name = 'test-app'
version = '0.1.0'

[component.mcp-gateway]
allowed_outbound_hosts = ['http://*.spin.internal']

[component.mcp-gateway.source]
package = 'fastertools:mcp-gateway'
registry = 'ghcr.io'
version = '0.0.13-alpha.0'

[trigger]
[[trigger.http]]
component = 'mcp-gateway'
route = '/...'
`

		if outputFile != "" {
			// Write to the specified file
			if err := os.WriteFile(outputFile, []byte(spinTOML), 0644); err != nil {
				fmt.Fprintf(os.Stderr, "Error writing file: %v\n", err)
				os.Exit(1)
			}
			fmt.Printf("Generated %s\n", outputFile)
		} else {
			// Output to stdout
			fmt.Print(spinTOML)
		}
	case "build":
		fmt.Println("Building FTL application...")
		fmt.Println("✓ Built successfully")
	case "deploy":
		fmt.Println("Deploying FTL application...")
		fmt.Println("✓ Deployed successfully")
	default:
		fmt.Printf("Unknown ftl command: %s\n", args[0])
		os.Exit(1)
	}
}
