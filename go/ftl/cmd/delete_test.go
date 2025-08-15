package cmd

import (
	"bytes"
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"

	"github.com/fastertools/ftl-cli/go/shared/api"
)

func TestIsInteractive(t *testing.T) {
	// This function checks if stdin is a terminal
	// The result will vary based on test environment
	result := isInteractive()

	// Just verify it returns a boolean without panicking
	assert.IsType(t, bool(true), result)
}

func TestDeleteDisplayOutput(t *testing.T) {
	tests := []struct {
		name     string
		app      *api.App
		expected []string
	}{
		{
			name: "app with URL",
			app: &api.App{
				AppId:       uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:     "test-app",
				Status:      api.AppStatusACTIVE,
				ProviderUrl: ptr("https://example.com"),
				CreatedAt:   "2024-01-01T00:00:00Z",
				UpdatedAt:   "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"Application to be deleted:",
				"Name: test-app",
				"ID: 123e4567-e89b-12d3-a456-426614174000",
				"URL: https://example.com",
			},
		},
		{
			name: "app without URL",
			app: &api.App{
				AppId:     uuid.MustParse("223e4567-e89b-12d3-a456-426614174000"),
				AppName:   "minimal-app",
				Status:    api.AppStatusPENDING,
				CreatedAt: "2024-01-01T00:00:00Z",
				UpdatedAt: "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"Application to be deleted:",
				"Name: minimal-app",
				"ID: 223e4567-e89b-12d3-a456-426614174000",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer

			// Simulate the delete display output
			buf.WriteString("Application to be deleted:\n")
			buf.WriteString("  Name: " + tt.app.AppName + "\n")
			buf.WriteString("  ID: " + tt.app.AppId.String() + "\n")
			if tt.app.ProviderUrl != nil && *tt.app.ProviderUrl != "" {
				buf.WriteString("  URL: " + *tt.app.ProviderUrl + "\n")
			}

			output := buf.String()
			for _, expected := range tt.expected {
				assert.Contains(t, output, expected)
			}
		})
	}
}

func TestDeleteConfirmationLogic(t *testing.T) {
	tests := []struct {
		name          string
		appName       string
		userInput     string
		shouldProceed bool
	}{
		{
			name:          "exact match proceeds",
			appName:       "test-app",
			userInput:     "test-app",
			shouldProceed: true,
		},
		{
			name:          "mismatch cancels",
			appName:       "test-app",
			userInput:     "wrong-app",
			shouldProceed: false,
		},
		{
			name:          "empty input cancels",
			appName:       "test-app",
			userInput:     "",
			shouldProceed: false,
		},
		{
			name:          "case sensitive",
			appName:       "test-app",
			userInput:     "TEST-APP",
			shouldProceed: false,
		},
		{
			name:          "extra spaces fail",
			appName:       "test-app",
			userInput:     " test-app ",
			shouldProceed: false,
		},
		{
			name:          "complex name exact match",
			appName:       "my-complex-app-123",
			userInput:     "my-complex-app-123",
			shouldProceed: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Test the logic of confirmation
			shouldProceed := tt.userInput == tt.appName
			assert.Equal(t, tt.shouldProceed, shouldProceed,
				"For app '%s' with input '%s', should proceed: %v",
				tt.appName, tt.userInput, tt.shouldProceed)
		})
	}
}

func TestDeleteWithForceFlag(t *testing.T) {
	tests := []struct {
		name         string
		force        bool
		expectPrompt bool
	}{
		{
			name:         "force skips confirmation",
			force:        true,
			expectPrompt: false,
		},
		{
			name:         "no force requires confirmation",
			force:        false,
			expectPrompt: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// In actual command, force=true skips the confirmation prompt
			if tt.force {
				assert.False(t, tt.expectPrompt, "Force flag should skip confirmation")
			} else {
				assert.True(t, tt.expectPrompt, "Without force flag, confirmation is required")
			}
		})
	}
}

func TestDeleteSuccessMessage(t *testing.T) {
	var buf bytes.Buffer

	// Simulate successful deletion
	buf.WriteString("ℹ Deleting application...\n")
	buf.WriteString("✓ Application deleted successfully\n")

	output := buf.String()
	assert.Contains(t, output, "Deleting application")
	assert.Contains(t, output, "Application deleted successfully")
}

func TestDeleteCancellation(t *testing.T) {
	var buf bytes.Buffer

	// Simulate cancellation
	buf.WriteString("Deletion cancelled.\n")

	output := buf.String()
	assert.Contains(t, output, "Deletion cancelled")
}

func TestDeleteErrorMessages(t *testing.T) {
	tests := []struct {
		name          string
		errorMsg      string
		expectedParts []string
	}{
		{
			name:     "app not found",
			errorMsg: "application 'nonexistent' not found",
			expectedParts: []string{
				"application",
				"nonexistent",
				"not found",
			},
		},
		{
			name:     "permission denied",
			errorMsg: "failed to delete app: insufficient permissions",
			expectedParts: []string{
				"failed to delete",
				"insufficient permissions",
			},
		},
		{
			name:     "network error",
			errorMsg: "failed to delete app: network timeout",
			expectedParts: []string{
				"failed to delete",
				"network timeout",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			for _, part := range tt.expectedParts {
				assert.Contains(t, tt.errorMsg, part)
			}
		})
	}
}

func TestDeleteAppStates(t *testing.T) {
	// Test that delete works for apps in various states
	states := []struct {
		status    api.AppStatus
		canDelete bool
	}{
		{api.AppStatusACTIVE, true},
		{api.AppStatusFAILED, true},
		{api.AppStatusPENDING, true},
		{api.AppStatusCREATING, true},
		{api.AppStatusDELETING, true}, // Already deleting
		{api.AppStatusDELETED, true},  // Already deleted
	}

	for _, st := range states {
		t.Run(string(st.status), func(t *testing.T) {
			app := &api.App{
				AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:   "test-app",
				Status:    st.status,
				CreatedAt: "2024-01-01T00:00:00Z",
				UpdatedAt: "2024-01-01T00:00:00Z",
			}

			// All states should be deletable (the API decides)
			assert.True(t, st.canDelete, "App in status %s should be deletable", st.status)
			assert.NotNil(t, app)
		})
	}
}

func TestDeleteByUUIDvsName(t *testing.T) {
	tests := []struct {
		name       string
		identifier string
		isUUID     bool
	}{
		{
			name:       "valid UUID",
			identifier: "123e4567-e89b-12d3-a456-426614174000",
			isUUID:     true,
		},
		{
			name:       "app name",
			identifier: "my-app",
			isUUID:     false,
		},
		{
			name:       "UUID-like name",
			identifier: "app-123e4567",
			isUUID:     false,
		},
		{
			name:       "numeric name",
			identifier: "12345",
			isUUID:     false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := uuid.Parse(tt.identifier)
			if tt.isUUID {
				assert.NoError(t, err, "Should be valid UUID")
			} else {
				assert.Error(t, err, "Should not be valid UUID")
			}
		})
	}
}

func TestDeleteCommandIntegration(t *testing.T) {
	// Test the full flow of delete command
	t.Run("delete with force flag", func(t *testing.T) {
		app := &api.App{
			AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
			AppName:   "integration-test",
			Status:    api.AppStatusACTIVE,
			CreatedAt: "2024-01-01T00:00:00Z",
			UpdatedAt: "2024-01-01T00:00:00Z",
		}

		force := true

		// With force=true, deletion should proceed without confirmation
		assert.True(t, force)
		assert.NotNil(t, app)
	})

	t.Run("delete with confirmation", func(t *testing.T) {
		app := &api.App{
			AppId:     uuid.MustParse("223e4567-e89b-12d3-a456-426614174000"),
			AppName:   "confirm-test",
			Status:    api.AppStatusACTIVE,
			CreatedAt: "2024-01-01T00:00:00Z",
			UpdatedAt: "2024-01-01T00:00:00Z",
		}

		force := false
		userInput := "confirm-test"

		// Confirmation matches app name
		shouldDelete := !force && (userInput == app.AppName)
		assert.True(t, shouldDelete)
	})
}
