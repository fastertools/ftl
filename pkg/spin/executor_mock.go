package spin

import (
	"context"
	"fmt"
	"io"
	"strings"
)

// MockExecutor is a mock implementation of the Executor interface for testing
type MockExecutor struct {
	// Configuration for mock behavior
	RunFunc           func(ctx context.Context, args ...string) error
	RunWithOutputFunc func(ctx context.Context, args ...string) (string, error)
	RunWithInputFunc  func(ctx context.Context, input io.Reader, args ...string) error
	RunInteractiveFunc func(ctx context.Context, args ...string) error
	IsInstalledFunc   func() bool
	VersionFunc       func() (string, error)
	
	// Track calls for assertions
	Calls []MockCall
}

// MockCall records a method call for verification
type MockCall struct {
	Method string
	Args   []string
}

// NewMockExecutor creates a new mock executor with sensible defaults
func NewMockExecutor() *MockExecutor {
	return &MockExecutor{
		RunFunc: func(ctx context.Context, args ...string) error {
			return nil
		},
		RunWithOutputFunc: func(ctx context.Context, args ...string) (string, error) {
			return "", nil
		},
		RunWithInputFunc: func(ctx context.Context, input io.Reader, args ...string) error {
			return nil
		},
		RunInteractiveFunc: func(ctx context.Context, args ...string) error {
			return nil
		},
		IsInstalledFunc: func() bool {
			return true
		},
		VersionFunc: func() (string, error) {
			return "2.0.0", nil
		},
	}
}

// Run implements Executor
func (m *MockExecutor) Run(ctx context.Context, args ...string) error {
	m.Calls = append(m.Calls, MockCall{Method: "Run", Args: args})
	if m.RunFunc != nil {
		return m.RunFunc(ctx, args...)
	}
	return nil
}

// RunWithOutput implements Executor
func (m *MockExecutor) RunWithOutput(ctx context.Context, args ...string) (string, error) {
	m.Calls = append(m.Calls, MockCall{Method: "RunWithOutput", Args: args})
	if m.RunWithOutputFunc != nil {
		return m.RunWithOutputFunc(ctx, args...)
	}
	return "", nil
}

// RunWithInput implements Executor
func (m *MockExecutor) RunWithInput(ctx context.Context, input io.Reader, args ...string) error {
	m.Calls = append(m.Calls, MockCall{Method: "RunWithInput", Args: args})
	if m.RunWithInputFunc != nil {
		return m.RunWithInputFunc(ctx, input, args...)
	}
	return nil
}

// RunInteractive implements Executor
func (m *MockExecutor) RunInteractive(ctx context.Context, args ...string) error {
	m.Calls = append(m.Calls, MockCall{Method: "RunInteractive", Args: args})
	if m.RunInteractiveFunc != nil {
		return m.RunInteractiveFunc(ctx, args...)
	}
	return nil
}

// IsInstalled implements Executor
func (m *MockExecutor) IsInstalled() bool {
	m.Calls = append(m.Calls, MockCall{Method: "IsInstalled", Args: nil})
	if m.IsInstalledFunc != nil {
		return m.IsInstalledFunc()
	}
	return true
}

// Version implements Executor
func (m *MockExecutor) Version() (string, error) {
	m.Calls = append(m.Calls, MockCall{Method: "Version", Args: nil})
	if m.VersionFunc != nil {
		return m.VersionFunc()
	}
	return "2.0.0", nil
}

// AssertCalled verifies a method was called with specific arguments
func (m *MockExecutor) AssertCalled(method string, args ...string) error {
	for _, call := range m.Calls {
		if call.Method == method {
			if len(args) == 0 || equalArgs(call.Args, args) {
				return nil
			}
		}
	}
	return fmt.Errorf("method %s was not called with args %v", method, args)
}

func equalArgs(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// Reset clears all recorded calls
func (m *MockExecutor) Reset() {
	m.Calls = nil
}

// CallCount returns the number of times a method was called
func (m *MockExecutor) CallCount(method string) int {
	count := 0
	for _, call := range m.Calls {
		if call.Method == method {
			count++
		}
	}
	return count
}

// LastCall returns the most recent call for a given method
func (m *MockExecutor) LastCall(method string) *MockCall {
	for i := len(m.Calls) - 1; i >= 0; i-- {
		if m.Calls[i].Method == method {
			return &m.Calls[i]
		}
	}
	return nil
}

// SetVersionOutput sets the mock version output
func (m *MockExecutor) SetVersionOutput(version string) {
	m.VersionFunc = func() (string, error) {
		return version, nil
	}
}

// SetRunError makes Run return an error
func (m *MockExecutor) SetRunError(err error) {
	m.RunFunc = func(ctx context.Context, args ...string) error {
		return err
	}
}

// SetRunWithOutputResponse sets the response for RunWithOutput
func (m *MockExecutor) SetRunWithOutputResponse(output string, err error) {
	m.RunWithOutputFunc = func(ctx context.Context, args ...string) (string, error) {
		return output, err
	}
}

// SimulateCommandOutput simulates output based on the command
func (m *MockExecutor) SimulateCommandOutput(command string) {
	m.RunWithOutputFunc = func(ctx context.Context, args ...string) (string, error) {
		if len(args) == 0 {
			return "", fmt.Errorf("no command provided")
		}
		
		// Simulate some common spin commands
		switch args[0] {
		case "--version":
			return "spin 2.0.0 (2.0.0 2024-01-01)", nil
		case "build":
			return "Building Spin application...\nBuild complete", nil
		case "up":
			return "Starting Spin application on http://127.0.0.1:3000", nil
		case "deploy":
			return "Deploying application...\nDeployment successful", nil
		default:
			return "", fmt.Errorf("unknown command: %s", strings.Join(args, " "))
		}
	}
}