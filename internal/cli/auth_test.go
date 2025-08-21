package cli

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestAuthCommand(t *testing.T) {
	cmd := newAuthCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "auth", cmd.Use)
	assert.Contains(t, cmd.Short, "Manage authentication")

	// Test subcommands exist
	expectedSubcommands := []string{"login", "logout", "status"}
	for _, subcmd := range expectedSubcommands {
		t.Run("has_"+subcmd, func(t *testing.T) {
			found := false
			for _, c := range cmd.Commands() {
				if c.Name() == subcmd {
					found = true
					break
				}
			}
			assert.True(t, found, "Subcommand %s not found", subcmd)
		})
	}
}

func TestAuthLoginCommand(t *testing.T) {
	cmd := newAuthLoginCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "login", cmd.Use)
	assert.Contains(t, cmd.Short, "Login")

	// Test flags
	noBrowserFlag := cmd.Flags().Lookup("no-browser")
	assert.NotNil(t, noBrowserFlag)
	assert.Equal(t, "false", noBrowserFlag.DefValue)

	forceFlag := cmd.Flags().Lookup("force")
	assert.NotNil(t, forceFlag)
	assert.Equal(t, "false", forceFlag.DefValue)

	authDomainFlag := cmd.Flags().Lookup("auth-domain")
	assert.NotNil(t, authDomainFlag)
}

func TestAuthLogoutCommand(t *testing.T) {
	cmd := newAuthLogoutCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "logout", cmd.Use)
	assert.Contains(t, cmd.Short, "Logout")

	// Logout command has no flags
	assert.Equal(t, 0, cmd.Flags().NFlag())
}

func TestAuthStatusCommand(t *testing.T) {
	cmd := newAuthStatusCmd()

	// Test command structure
	assert.NotNil(t, cmd)
	assert.Equal(t, "status", cmd.Use)
	assert.Contains(t, cmd.Short, "authentication status")

	// Test flags
	showTokenFlag := cmd.Flags().Lookup("show-token")
	assert.NotNil(t, showTokenFlag)
	assert.Equal(t, "false", showTokenFlag.DefValue)
}

func TestAuthCommand_Help(t *testing.T) {
	cmd := newAuthCmd()
	var buf bytes.Buffer
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"--help"})

	err := cmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "auth")
	assert.Contains(t, output, "Available Commands:")
	assert.Contains(t, output, "login")
	assert.Contains(t, output, "logout")
	assert.Contains(t, output, "status")
}

func TestAuthLoginCommand_Help(t *testing.T) {
	// Create auth command with login subcommand
	authCmd := newAuthCmd()

	var buf bytes.Buffer
	authCmd.SetOut(&buf)
	authCmd.SetArgs([]string{"login", "--help"})

	err := authCmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "login")
	assert.Contains(t, output, "Flags:")
}

func TestAuthLogoutCommand_Help(t *testing.T) {
	// Create auth command with logout subcommand
	authCmd := newAuthCmd()

	var buf bytes.Buffer
	authCmd.SetOut(&buf)
	authCmd.SetArgs([]string{"logout", "--help"})

	err := authCmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "logout")
}

func TestAuthStatusCommand_Help(t *testing.T) {
	// Create auth command with status subcommand
	authCmd := newAuthCmd()

	var buf bytes.Buffer
	authCmd.SetOut(&buf)
	authCmd.SetArgs([]string{"status", "--help"})

	err := authCmd.Execute()
	assert.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "Usage:")
	assert.Contains(t, output, "status")
}

func TestAuthLoginCommand_Flags(t *testing.T) {
	cmd := newAuthLoginCmd()

	// Test that all expected flags are present
	tests := []struct {
		flagName  string
		shorthand string
		defValue  string
	}{
		{"no-browser", "", "false"},
		{"force", "", "false"},
		{"auth-domain", "", ""},
	}

	for _, tt := range tests {
		t.Run(tt.flagName, func(t *testing.T) {
			flag := cmd.Flags().Lookup(tt.flagName)
			assert.NotNil(t, flag, "Flag %s should exist", tt.flagName)
			if tt.shorthand != "" {
				assert.Equal(t, tt.shorthand, flag.Shorthand)
			}
			if tt.defValue != "" {
				assert.Equal(t, tt.defValue, flag.DefValue)
			}
		})
	}
}

func TestAuthStatusCommand_Flags(t *testing.T) {
	cmd := newAuthStatusCmd()

	// Test show-token flag
	flag := cmd.Flags().Lookup("show-token")
	assert.NotNil(t, flag)
	assert.Equal(t, "false", flag.DefValue)
}

func BenchmarkAuthCommand(b *testing.B) {
	for i := 0; i < b.N; i++ {
		cmd := newAuthCmd()
		cmd.SetOut(&bytes.Buffer{})
		cmd.SetArgs([]string{"--help"})
		_ = cmd.Execute()
	}
}

func BenchmarkAuthLogin(b *testing.B) {
	for i := 0; i < b.N; i++ {
		authCmd := newAuthCmd()
		authCmd.SetOut(&bytes.Buffer{})
		authCmd.SetArgs([]string{"login", "--help"})
		_ = authCmd.Execute()
	}
}
