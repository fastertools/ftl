package cmd

import (
	"bytes"
	"testing"

	"github.com/fatih/color"
	"github.com/google/uuid"
	openapi_types "github.com/oapi-codegen/runtime/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/fastertools/ftl-cli/go/shared/api"
)

func TestDisplayAppsTable(t *testing.T) {
	// Disable color for consistent test output
	color.NoColor = true
	defer func() { color.NoColor = false }()

	tests := []struct {
		name     string
		apps     []testApp
		verbose  bool
		expected []string
		notWant  []string
	}{
		{
			name: "multiple apps with different statuses",
			apps: []testApp{
				{
					id:          "123e4567-e89b-12d3-a456-426614174000",
					name:        "active-app",
					status:      api.ListAppsResponseBodyAppsStatusACTIVE,
					providerUrl: ptr("https://example.com"),
					createdAt:   "2024-01-01T00:00:00Z",
					updatedAt:   "2024-01-01T00:00:00Z",
				},
				{
					id:        "223e4567-e89b-12d3-a456-426614174000",
					name:      "failed-app",
					status:    api.ListAppsResponseBodyAppsStatusFAILED,
					createdAt: "2024-01-01T00:00:00Z",
					updatedAt: "2024-01-01T00:00:00Z",
				},
				{
					id:        "323e4567-e89b-12d3-a456-426614174000",
					name:      "pending-app",
					status:    api.ListAppsResponseBodyAppsStatusPENDING,
					createdAt: "2024-01-01T00:00:00Z",
					updatedAt: "2024-01-01T00:00:00Z",
				},
			},
			verbose: false,
			expected: []string{
				"NAME", "STATUS", "ACCESS", "URL", "CREATED",
				"active-app", "ACTIVE", "https://example.com",
				"failed-app", "FAILED",
				"pending-app", "PENDING",
				"Total: 3 applications",
			},
		},
		{
			name: "single app",
			apps: []testApp{
				{
					id:        "123e4567-e89b-12d3-a456-426614174000",
					name:      "solo-app",
					status:    api.ListAppsResponseBodyAppsStatusACTIVE,
					createdAt: "2024-01-01T00:00:00Z",
					updatedAt: "2024-01-01T00:00:00Z",
				},
			},
			verbose: false,
			expected: []string{
				"NAME", "STATUS", "ACCESS", "URL", "CREATED",
				"solo-app", "ACTIVE",
				"Total: 1 application", // singular
			},
			notWant: []string{
				"Total: 1 applications", // should not be plural
			},
		},
		{
			name:     "empty list",
			apps:     []testApp{},
			verbose:  false,
			expected: []string{},
		},
		{
			name: "verbose mode with deployment",
			apps: []testApp{
				{
					id:            "123e4567-e89b-12d3-a456-426614174000",
					name:          "verbose-app",
					status:        api.ListAppsResponseBodyAppsStatusACTIVE,
					providerUrl:   ptr("https://example.com"),
					providerError: ptr("Some error occurred"),
					createdAt:     "2024-01-01T00:00:00Z",
					updatedAt:     "2024-01-01T12:00:00Z",
					hasDeployment: true,
					deploymentId:  "deploy-123",
					environment:   ptr("production"),
				},
			},
			verbose: true,
			expected: []string{
				"verbose-app",
				"ACTIVE",
				"https://example.com",
				"2024-01-01", // dates should be visible
				"deploy-123",
				"production",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			// Convert test data to actual API type
			apps := convertToAPIApps(tt.apps)

			dw := NewDataWriter(&buf, "table")
			err := displayAppsTable(apps, tt.verbose, dw)
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

// Test helper types
type testApp struct {
	id            string
	name          string
	status        api.ListAppsResponseBodyAppsStatus
	providerUrl   *string
	providerError *string
	createdAt     string
	updatedAt     string
	hasDeployment bool
	deploymentId  string
	environment   *string
}

// Helper function to convert test data to API type
func convertToAPIApps(testApps []testApp) []struct {
	AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
	AllowedRoles  *[]string                                  `json:"allowedRoles,omitempty"`
	AppId         openapi_types.UUID                         `json:"appId"`
	AppName       string                                     `json:"appName"`
	CreatedAt     string                                     `json:"createdAt"`
	CustomAuth    *struct {
		Audience string `json:"audience"`
		Issuer   string `json:"issuer"`
	} `json:"customAuth,omitempty"`
	LatestDeployment *struct {
		CreatedAt          *float32                                           `json:"createdAt,omitempty"`
		DeployedAt         *float32                                           `json:"deployedAt,omitempty"`
		DeploymentDuration *float32                                           `json:"deploymentDuration,omitempty"`
		DeploymentId       string                                             `json:"deploymentId"`
		Environment        *string                                            `json:"environment,omitempty"`
		Status             api.ListAppsResponseBodyAppsLatestDeploymentStatus `json:"status"`
		StatusMessage      *string                                            `json:"statusMessage,omitempty"`
	} `json:"latestDeployment"`
	OrgId         *string                            `json:"orgId,omitempty"`
	ProviderError *string                            `json:"providerError,omitempty"`
	ProviderUrl   *string                            `json:"providerUrl,omitempty"`
	Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
	UpdatedAt     string                             `json:"updatedAt"`
} {
	result := make([]struct {
		AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
		AllowedRoles  *[]string                                  `json:"allowedRoles,omitempty"`
		AppId         openapi_types.UUID                         `json:"appId"`
		AppName       string                                     `json:"appName"`
		CreatedAt     string                                     `json:"createdAt"`
		CustomAuth    *struct {
			Audience string `json:"audience"`
			Issuer   string `json:"issuer"`
		} `json:"customAuth,omitempty"`
		LatestDeployment *struct {
			CreatedAt          *float32                                           `json:"createdAt,omitempty"`
			DeployedAt         *float32                                           `json:"deployedAt,omitempty"`
			DeploymentDuration *float32                                           `json:"deploymentDuration,omitempty"`
			DeploymentId       string                                             `json:"deploymentId"`
			Environment        *string                                            `json:"environment,omitempty"`
			Status             api.ListAppsResponseBodyAppsLatestDeploymentStatus `json:"status"`
			StatusMessage      *string                                            `json:"statusMessage,omitempty"`
		} `json:"latestDeployment"`
		OrgId         *string                            `json:"orgId,omitempty"`
		ProviderError *string                            `json:"providerError,omitempty"`
		ProviderUrl   *string                            `json:"providerUrl,omitempty"`
		Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
		UpdatedAt     string                             `json:"updatedAt"`
	}, len(testApps))

	for i, app := range testApps {
		result[i].AppId = mustParseUUID(app.id)
		result[i].AppName = app.name
		result[i].Status = app.status
		result[i].ProviderUrl = app.providerUrl
		result[i].ProviderError = app.providerError
		result[i].CreatedAt = app.createdAt
		result[i].UpdatedAt = app.updatedAt

		// Add deployment info if present
		if app.hasDeployment {
			result[i].LatestDeployment = &struct {
				CreatedAt          *float32                                           `json:"createdAt,omitempty"`
				DeployedAt         *float32                                           `json:"deployedAt,omitempty"`
				DeploymentDuration *float32                                           `json:"deploymentDuration,omitempty"`
				DeploymentId       string                                             `json:"deploymentId"`
				Environment        *string                                            `json:"environment,omitempty"`
				Status             api.ListAppsResponseBodyAppsLatestDeploymentStatus `json:"status"`
				StatusMessage      *string                                            `json:"statusMessage,omitempty"`
			}{
				DeploymentId: app.deploymentId,
				Environment:  app.environment,
				Status:       api.ListAppsResponseBodyAppsLatestDeploymentStatusDeployed,
			}
		}
	}

	return result
}

// Test helper functions
func ptr(s string) *string {
	return &s
}

func mustParseUUID(s string) openapi_types.UUID {
	// Parse using google/uuid and convert to openapi_types.UUID
	googleUUID, err := uuid.Parse(s)
	if err != nil {
		panic(err)
	}
	return openapi_types.UUID(googleUUID)
}
