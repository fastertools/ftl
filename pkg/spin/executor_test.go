package spin

import (
	"context"
	"errors"
	"io"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestExecutor_Run_WithMock(t *testing.T) {
	t.Run("successful command", func(t *testing.T) {
		mock := NewMockExecutor()
		
		err := mock.Run(context.Background(), "build")
		assert.NoError(t, err)
		assert.Equal(t, 1, mock.CallCount("Run"))
		assert.NoError(t, mock.AssertCalled("Run", "build"))
	})

	t.Run("failed command", func(t *testing.T) {
		mock := NewMockExecutor()
		expectedErr := errors.New("build failed")
		mock.SetRunError(expectedErr)
		
		err := mock.Run(context.Background(), "build")
		assert.Error(t, err)
		assert.Equal(t, expectedErr, err)
	})

	t.Run("context cancellation", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.RunFunc = func(ctx context.Context, args ...string) error {
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
				return nil
			}
		}
		
		ctx, cancel := context.WithCancel(context.Background())
		cancel() // Cancel immediately
		
		err := mock.Run(ctx, "up")
		assert.Error(t, err)
		assert.Equal(t, context.Canceled, err)
	})
}

func TestExecutor_RunWithOutput_WithMock(t *testing.T) {
	t.Run("capture output", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.SetRunWithOutputResponse("Build complete\nApplication ready", nil)
		
		output, err := mock.RunWithOutput(context.Background(), "build")
		assert.NoError(t, err)
		assert.Equal(t, "Build complete\nApplication ready", output)
	})

	t.Run("error with output", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.SetRunWithOutputResponse("", errors.New("build failed"))
		
		output, err := mock.RunWithOutput(context.Background(), "build")
		assert.Error(t, err)
		assert.Empty(t, output)
	})
}

func TestExecutor_RunWithInput_WithMock(t *testing.T) {
	t.Run("basic flow", func(t *testing.T) {
		mock := NewMockExecutor()
		input := strings.NewReader("test input")
		
		err := mock.RunWithInput(context.Background(), input, "deploy")
		assert.NoError(t, err)
		assert.Equal(t, 1, mock.CallCount("RunWithInput"))
	})

	t.Run("with error", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.RunWithInputFunc = func(ctx context.Context, input io.Reader, args ...string) error {
			return errors.New("deployment failed")
		}
		
		input := strings.NewReader("test input")
		err := mock.RunWithInput(context.Background(), input, "deploy")
		assert.Error(t, err)
		assert.Equal(t, "deployment failed", err.Error())
	})
}

func TestExecutor_IsInstalled_WithMock(t *testing.T) {
	t.Run("installed", func(t *testing.T) {
		mock := NewMockExecutor()
		assert.True(t, mock.IsInstalled())
	})

	t.Run("not installed", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.IsInstalledFunc = func() bool {
			return false
		}
		assert.False(t, mock.IsInstalled())
	})
}

func TestExecutor_Version_WithMock(t *testing.T) {
	t.Run("parse version success", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.SetVersionOutput("2.5.0")
		
		version, err := mock.Version()
		assert.NoError(t, err)
		assert.Equal(t, "2.5.0", version)
	})

	t.Run("version error", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.VersionFunc = func() (string, error) {
			return "", errors.New("failed to get version")
		}
		
		version, err := mock.Version()
		assert.Error(t, err)
		assert.Empty(t, version)
	})
}

func TestExecutor_RealImplementation_ValidationOnly(t *testing.T) {
	// These tests only verify that our validation works correctly
	// They don't actually execute anything
	
	t.Run("accepts spin binary", func(t *testing.T) {
		e := NewExecutor(WithBinary("spin"))
		executor := e.(*executor)
		assert.Equal(t, "spin", executor.binary)
	})
	
	t.Run("accepts absolute path to spin", func(t *testing.T) {
		e := NewExecutor(WithBinary("/usr/local/bin/spin"))
		executor := e.(*executor)
		assert.Equal(t, "/usr/local/bin/spin", executor.binary)
	})
	
	t.Run("rejects non-spin binary", func(t *testing.T) {
		e := NewExecutor(WithBinary("echo"))
		executor := e.(*executor)
		assert.Equal(t, "spin", executor.binary) // Should keep default
	})
	
	t.Run("rejects relative non-spin path", func(t *testing.T) {
		e := NewExecutor(WithBinary("./not-spin"))
		executor := e.(*executor)
		assert.Equal(t, "spin", executor.binary) // Should keep default
	})
	
	t.Run("rejects absolute path to non-spin binary", func(t *testing.T) {
		e := NewExecutor(WithBinary("/usr/bin/echo"))
		executor := e.(*executor)
		assert.Equal(t, "spin", executor.binary) // Should keep default
	})
}

func TestMockExecutor_Helpers(t *testing.T) {
	t.Run("tracks multiple calls", func(t *testing.T) {
		mock := NewMockExecutor()
		
		_ = mock.Run(context.Background(), "build")
		_ = mock.Run(context.Background(), "up")
		_, _ = mock.RunWithOutput(context.Background(), "--version")
		
		assert.Equal(t, 2, mock.CallCount("Run"))
		assert.Equal(t, 1, mock.CallCount("RunWithOutput"))
		
		lastRun := mock.LastCall("Run")
		require.NotNil(t, lastRun)
		assert.Equal(t, []string{"up"}, lastRun.Args)
	})
	
	t.Run("reset clears calls", func(t *testing.T) {
		mock := NewMockExecutor()
		
		_ = mock.Run(context.Background(), "build")
		assert.Equal(t, 1, len(mock.Calls))
		
		mock.Reset()
		assert.Equal(t, 0, len(mock.Calls))
	})
	
	t.Run("simulate command output", func(t *testing.T) {
		mock := NewMockExecutor()
		mock.SimulateCommandOutput("spin")
		
		output, err := mock.RunWithOutput(context.Background(), "--version")
		assert.NoError(t, err)
		assert.Contains(t, output, "spin 2.0.0")
		
		output, err = mock.RunWithOutput(context.Background(), "build")
		assert.NoError(t, err)
		assert.Contains(t, output, "Building Spin application")
		
		output, err = mock.RunWithOutput(context.Background(), "unknown-command")
		assert.Error(t, err)
		assert.Empty(t, output)
	})
}