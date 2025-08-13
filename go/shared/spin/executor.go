// Package spin provides utilities for orchestrating Spin CLI commands
package spin

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"time"

	"github.com/pkg/errors"
)

// Executor provides an interface for executing Spin commands
type Executor interface {
	Run(ctx context.Context, args ...string) error
	RunWithOutput(ctx context.Context, args ...string) (string, error)
	RunWithInput(ctx context.Context, input string, args ...string) error
	RunInteractive(ctx context.Context, args ...string) error
	IsInstalled() bool
	Version() (string, error)
}

// executor is the default implementation of Executor
type executor struct {
	binary string
	env    []string
	dir    string
	stdout io.Writer
	stderr io.Writer
	stdin  io.Reader
}

// NewExecutor creates a new Spin executor
func NewExecutor(options ...Option) Executor {
	e := &executor{
		binary: "spin",
		stdout: os.Stdout,
		stderr: os.Stderr,
		stdin:  os.Stdin,
	}

	for _, opt := range options {
		opt(e)
	}

	return e
}

// Option configures an executor
type Option func(*executor)

// WithBinary sets the Spin binary path
func WithBinary(binary string) Option {
	return func(e *executor) {
		e.binary = binary
	}
}

// WithEnv sets environment variables
func WithEnv(env []string) Option {
	return func(e *executor) {
		e.env = env
	}
}

// WithDir sets the working directory
func WithDir(dir string) Option {
	return func(e *executor) {
		e.dir = dir
	}
}

// WithOutput sets custom output writers
func WithOutput(stdout, stderr io.Writer) Option {
	return func(e *executor) {
		e.stdout = stdout
		e.stderr = stderr
	}
}

// WithInput sets custom input reader
func WithInput(stdin io.Reader) Option {
	return func(e *executor) {
		e.stdin = stdin
	}
}

// Run executes a Spin command
func (e *executor) Run(ctx context.Context, args ...string) error {
	cmd := e.command(ctx, args...)
	cmd.Stdout = e.stdout
	cmd.Stderr = e.stderr
	return e.runCommand(cmd)
}

// RunWithOutput executes a Spin command and returns output
func (e *executor) RunWithOutput(ctx context.Context, args ...string) (string, error) {
	cmd := e.command(ctx, args...)
	var stdout bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = e.stderr
	
	if err := e.runCommand(cmd); err != nil {
		return "", err
	}
	
	return strings.TrimSpace(stdout.String()), nil
}

// RunWithInput executes a Spin command with input
func (e *executor) RunWithInput(ctx context.Context, input string, args ...string) error {
	cmd := e.command(ctx, args...)
	cmd.Stdin = strings.NewReader(input)
	cmd.Stdout = e.stdout
	cmd.Stderr = e.stderr
	return e.runCommand(cmd)
}

// RunInteractive executes a Spin command interactively
func (e *executor) RunInteractive(ctx context.Context, args ...string) error {
	cmd := e.command(ctx, args...)
	cmd.Stdin = e.stdin
	cmd.Stdout = e.stdout
	cmd.Stderr = e.stderr
	return e.runCommand(cmd)
}

// IsInstalled checks if Spin is installed
func (e *executor) IsInstalled() bool {
	cmd := exec.Command(e.binary, "--version")
	return cmd.Run() == nil
}

// Version returns the Spin version
func (e *executor) Version() (string, error) {
	cmd := exec.Command(e.binary, "--version")
	output, err := cmd.Output()
	if err != nil {
		return "", errors.Wrap(err, "failed to get Spin version")
	}
	
	// Parse version from output like "spin 2.0.0 (2.0.0 2024-01-01)"
	parts := strings.Fields(string(output))
	if len(parts) < 2 {
		return "", fmt.Errorf("unexpected version output: %s", output)
	}
	
	return parts[1], nil
}

// command creates a new command
func (e *executor) command(ctx context.Context, args ...string) *exec.Cmd {
	cmd := exec.CommandContext(ctx, e.binary, args...)
	
	if e.dir != "" {
		cmd.Dir = e.dir
	}
	
	if len(e.env) > 0 {
		cmd.Env = append(os.Environ(), e.env...)
	}
	
	return cmd
}

// runCommand runs a command with error handling
func (e *executor) runCommand(cmd *exec.Cmd) error {
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			return errors.Wrapf(err, "spin command failed with exit code %d", exitErr.ExitCode())
		}
		return errors.Wrap(err, "failed to execute spin command")
	}
	return nil
}

// Common Spin commands as convenience functions

// Up runs 'spin up'
func Up(ctx context.Context, options ...string) error {
	args := append([]string{"up"}, options...)
	return NewExecutor().Run(ctx, args...)
}

// Build runs 'spin build'
func Build(ctx context.Context, options ...string) error {
	args := append([]string{"build"}, options...)
	return NewExecutor().Run(ctx, args...)
}

// Deploy runs 'spin deploy'
func Deploy(ctx context.Context, options ...string) error {
	args := append([]string{"deploy"}, options...)
	return NewExecutor().Run(ctx, args...)
}

// New runs 'spin new'
func New(ctx context.Context, template, name string, options ...string) error {
	args := append([]string{"new", template, name}, options...)
	return NewExecutor().Run(ctx, args...)
}

// Registry runs 'spin registry' commands
func Registry(ctx context.Context, subcommand string, options ...string) error {
	args := append([]string{"registry", subcommand}, options...)
	return NewExecutor().Run(ctx, args...)
}

// Watch runs 'spin watch'
func Watch(ctx context.Context, options ...string) error {
	args := append([]string{"watch"}, options...)
	return NewExecutor().Run(ctx, args...)
}

// EnsureInstalled ensures Spin is installed
func EnsureInstalled() error {
	e := NewExecutor()
	if !e.IsInstalled() {
		return errors.New("spin is not installed. Please install it from https://developer.fermyon.com/spin/install")
	}
	
	version, err := e.Version()
	if err != nil {
		return errors.Wrap(err, "failed to check Spin version")
	}
	
	// Check minimum version (2.0.0)
	if !isVersionSupported(version) {
		return fmt.Errorf("spin version %s is not supported, please upgrade to 2.0.0 or later", version)
	}
	
	return nil
}

// isVersionSupported checks if the version meets minimum requirements
func isVersionSupported(version string) bool {
	// Simple check - in production, use proper semver comparison
	return strings.HasPrefix(version, "2.") || strings.HasPrefix(version, "3.")
}

// WaitForReady waits for Spin to be ready on a given address
func WaitForReady(ctx context.Context, address string, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	
	for time.Now().Before(deadline) {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			// Try to connect (simplified - in production use proper HTTP client)
			cmd := exec.CommandContext(ctx, "curl", "-s", "-o", "/dev/null", "-w", "%{http_code}", address)
			if output, err := cmd.Output(); err == nil && string(output) == "200" {
				return nil
			}
			time.Sleep(100 * time.Millisecond)
		}
	}
	
	return errors.New("timeout waiting for Spin to be ready")
}