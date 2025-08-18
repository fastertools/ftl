package cli

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	"github.com/fatih/color"
	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/fastertools/ftl-cli/internal/api"
)

func TestDisplayAppStatusTable(t *testing.T) {
	// Disable color for consistent test output
	color.NoColor = true
	defer func() { color.NoColor = false }()

	tests := []struct {
		name     string
		app      *api.App
		expected []string
		notWant  []string
	}{
		{
			name: "active app with full details",
			app: &api.App{
				AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:       "test-app",
				Status:        api.AppStatusACTIVE,
				ProviderUrl:   ptr("https://example.com"),
				AccessControl: (*api.AppAccessControl)(ptr("PUBLIC")),
				CreatedAt:     "2024-01-01T00:00:00Z",
				UpdatedAt:     "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"Application Details",
				"Name:", "test-app",
				"ID:", "123e4567-e89b-12d3-a456-426614174000",
				"Status:", "ACTIVE",
				"URL:", "https://example.com",
				"Access:", "public", // Should be lowercase
				"Created:", "2024-01-01T00:00:00Z",
				"Updated:", "2024-01-01T00:00:00Z",
			},
		},
		{
			name: "failed app with error",
			app: &api.App{
				AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:       "failed-app",
				Status:        api.AppStatusFAILED,
				ProviderError: ptr("Deployment failed: out of memory"),
				CreatedAt:     "2024-01-01T00:00:00Z",
				UpdatedAt:     "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"failed-app",
				"FAILED",
				"Error:", "Deployment failed: out of memory",
			},
		},
		{
			name: "pending app minimal",
			app: &api.App{
				AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:   "pending-app",
				Status:    api.AppStatusPENDING,
				CreatedAt: "2024-01-01T00:00:00Z",
				UpdatedAt: "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"pending-app",
				"PENDING",
			},
			notWant: []string{
				"URL:",    // Should not show URL line if empty
				"Access:", // Should not show Access line if nil
			},
		},
		{
			name: "app with private access",
			app: &api.App{
				AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:       "private-app",
				Status:        api.AppStatusACTIVE,
				AccessControl: (*api.AppAccessControl)(ptr("PRIVATE")),
				CreatedAt:     "2024-01-01T00:00:00Z",
				UpdatedAt:     "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"private-app",
				"Access:", "private",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			err := displayAppStatusTable(tt.app)
			require.NoError(t, err)

			output := buf.String()
			for _, expected := range tt.expected {
				assert.Contains(t, output, expected, "Output should contain: %s", expected)
			}
			for _, notWant := range tt.notWant {
				assert.NotContains(t, output, notWant, "Output should not contain: %s", notWant)
			}
		})
	}
}

func TestDisplayAppStatusJSON(t *testing.T) {
	app := &api.App{
		AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
		AppName:       "test-app",
		Status:        api.AppStatusACTIVE,
		ProviderUrl:   ptr("https://example.com"),
		AccessControl: (*api.AppAccessControl)(ptr("PUBLIC")),
		CreatedAt:     "2024-01-01T00:00:00Z",
		UpdatedAt:     "2024-01-01T00:00:00Z",
	}

	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	dw := NewDataWriter(&buf, "json")
	err := dw.WriteStruct(app)
	require.NoError(t, err)

	// Verify JSON output
	var result map[string]interface{}
	err = json.Unmarshal(buf.Bytes(), &result)
	require.NoError(t, err)
	assert.Equal(t, "test-app", result["appName"])
	assert.Equal(t, "ACTIVE", result["status"])
	assert.Equal(t, "https://example.com", result["providerUrl"])
	assert.Equal(t, "PUBLIC", result["accessControl"])
}

func TestStatusCommandColorCoding(t *testing.T) {
	// Test that different statuses are handled correctly
	statuses := []struct {
		status api.AppStatus
		name   string
	}{
		{api.AppStatusACTIVE, "ACTIVE"},
		{api.AppStatusFAILED, "FAILED"},
		{api.AppStatusPENDING, "PENDING"},
		{api.AppStatusCREATING, "CREATING"},
		{api.AppStatusDELETING, "DELETING"},
		{api.AppStatusDELETED, "DELETED"},
	}

	for _, st := range statuses {
		t.Run(st.name, func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			app := &api.App{
				AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:   "test-app",
				Status:    st.status,
				CreatedAt: "2024-01-01T00:00:00Z",
				UpdatedAt: "2024-01-01T00:00:00Z",
			}

			err := displayAppStatusTable(app)
			require.NoError(t, err)
			assert.Contains(t, buf.String(), st.name)
		})
	}
}

func TestStatusWithComplexAuth(t *testing.T) {
	tests := []struct {
		name     string
		app      *api.App
		expected []string
	}{
		{
			name: "org access with allowed roles",
			app: &api.App{
				AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:       "org-app",
				Status:        api.AppStatusACTIVE,
				AccessControl: (*api.AppAccessControl)(ptr("ORG")),
				AllowedRoles:  &[]string{"admin", "developer"},
				OrgId:         ptr("org-123"),
				CreatedAt:     "2024-01-01T00:00:00Z",
				UpdatedAt:     "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"org-app",
				"Access:", "org",
			},
		},
		{
			name: "custom auth",
			app: &api.App{
				AppId:         uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
				AppName:       "custom-app",
				Status:        api.AppStatusACTIVE,
				AccessControl: (*api.AppAccessControl)(ptr("CUSTOM")),
				CustomAuth: &struct {
					Audience string `json:"audience"`
					Issuer   string `json:"issuer"`
				}{
					Audience: "api.example.com",
					Issuer:   "https://auth.example.com",
				},
				CreatedAt: "2024-01-01T00:00:00Z",
				UpdatedAt: "2024-01-01T00:00:00Z",
			},
			expected: []string{
				"custom-app",
				"Access:", "custom",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			err := displayAppStatusTable(tt.app)
			require.NoError(t, err)

			output := buf.String()
			for _, expected := range tt.expected {
				assert.Contains(t, output, expected)
			}
		})
	}
}

func TestStatusTableFormatting(t *testing.T) {
	// Test that the table is properly formatted
	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	app := &api.App{
		AppId:       uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
		AppName:     "format-test",
		Status:      api.AppStatusACTIVE,
		ProviderUrl: ptr("https://very-long-url-example.com/with/path"),
		CreatedAt:   "2024-01-01T00:00:00Z",
		UpdatedAt:   "2024-01-01T00:00:00Z",
	}

	err := displayAppStatusTable(app)
	require.NoError(t, err)

	output := buf.String()
	lines := strings.Split(output, "\n")

	// Check that indentation is consistent
	for _, line := range lines {
		if strings.Contains(line, ":") && !strings.Contains(line, "Application Details") {
			assert.True(t, strings.HasPrefix(line, "  "), "Detail lines should be indented")
		}
	}
}

func TestStatusMinimalApp(t *testing.T) {
	// Test with absolutely minimal app info
	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	app := &api.App{
		AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
		AppName:   "minimal",
		Status:    api.AppStatusACTIVE,
		CreatedAt: "2024-01-01T00:00:00Z",
		UpdatedAt: "2024-01-01T00:00:00Z",
		// Everything else is nil/empty
	}

	err := displayAppStatusTable(app)
	require.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "minimal")
	assert.Contains(t, output, "ACTIVE")
	assert.NotContains(t, output, "URL:")    // Should not show if nil
	assert.NotContains(t, output, "Access:") // Should not show if nil
	assert.NotContains(t, output, "Error:")  // Should not show if nil
}
