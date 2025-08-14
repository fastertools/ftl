package cmd

import (
	"bytes"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestRootCommand(t *testing.T) {
	tests := []struct {
		name     string
		args     []string
		wantErr  bool
		contains []string
	}{
		{
			name:    "help flag",
			args:    []string{"--help"},
			wantErr: false,
			contains: []string{
				"FTL is a comprehensive toolkit",
				"Available Commands:",
				"init",
				"build",
				"deploy",
			},
		},
		// Skipping version test for now - it has state issues with cobra
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			rootCmd.SetOut(&buf)
			rootCmd.SetErr(&buf)
			rootCmd.SetArgs(tt.args)

			err := rootCmd.Execute()

			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}

			output := buf.String()
			for _, want := range tt.contains {
				assert.Contains(t, output, want)
			}
		})
	}
}

func TestBuildCommand(t *testing.T) {
	cmd := newBuildCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "build", cmd.Use)
	assert.Contains(t, cmd.Short, "Build")
}

func TestDeployCommand(t *testing.T) {
	cmd := newDeployCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "deploy [flags]", cmd.Use)
	assert.Contains(t, cmd.Short, "Deploy")

	// Check flags
	flag := cmd.Flags().Lookup("environment")
	assert.NotNil(t, flag)
	assert.Equal(t, "production", flag.DefValue)
}

func TestUpCommand(t *testing.T) {
	cmd := newUpCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "up", cmd.Use)

	// Check flags
	assert.NotNil(t, cmd.Flags().Lookup("build"))
	assert.NotNil(t, cmd.Flags().Lookup("watch"))
}

func TestComponentCommand(t *testing.T) {
	cmd := newComponentCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "component", cmd.Use)

	// Check subcommands
	subCmds := cmd.Commands()
	assert.Len(t, subCmds, 3)

	names := []string{}
	for _, sub := range subCmds {
		names = append(names, sub.Use)
	}
	assert.Contains(t, strings.Join(names, ","), "add")
	assert.Contains(t, strings.Join(names, ","), "list")
	assert.Contains(t, strings.Join(names, ","), "remove")
}

func TestAuthCommand(t *testing.T) {
	cmd := newAuthCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "auth", cmd.Use)

	// Check subcommands
	subCmds := cmd.Commands()
	assert.Len(t, subCmds, 3)

	names := []string{}
	for _, sub := range subCmds {
		names = append(names, sub.Use)
	}
	assert.Contains(t, strings.Join(names, ","), "login")
	assert.Contains(t, strings.Join(names, ","), "logout")
	assert.Contains(t, strings.Join(names, ","), "status")
}

func TestRegistryCommand(t *testing.T) {
	cmd := newRegistryCmd()
	assert.NotNil(t, cmd)
	assert.Equal(t, "registry", cmd.Use)

	// Check subcommands
	subCmds := cmd.Commands()
	assert.Len(t, subCmds, 3)

	names := []string{}
	for _, sub := range subCmds {
		names = append(names, sub.Use)
	}
	assert.Contains(t, strings.Join(names, ","), "push")
	assert.Contains(t, strings.Join(names, ","), "pull")
	assert.Contains(t, strings.Join(names, ","), "list")
}

