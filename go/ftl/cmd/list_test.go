package cmd

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	"github.com/fatih/color"
	"github.com/google/uuid"
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
		apps     []appItem
		expected []string
		notWant  []string
	}{
		{
			name: "multiple apps with different statuses",
			apps: []appItem{
				{
					AppId:       uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
					AppName:     "active-app",
					Status:      api.ListAppsResponseBodyAppsStatusACTIVE,
					ProviderUrl: ptr("https://example.com"),
					CreatedAt:   "2024-01-01T00:00:00Z",
					UpdatedAt:   "2024-01-01T00:00:00Z",
				},
				{
					AppId:     uuid.MustParse("223e4567-e89b-12d3-a456-426614174000"),
					AppName:   "failed-app",
					Status:    api.ListAppsResponseBodyAppsStatusFAILED,
					CreatedAt: "2024-01-01T00:00:00Z",
					UpdatedAt: "2024-01-01T00:00:00Z",
				},
				{
					AppId:     uuid.MustParse("323e4567-e89b-12d3-a456-426614174000"),
					AppName:   "pending-app",
					Status:    api.ListAppsResponseBodyAppsStatusPENDING,
					CreatedAt: "2024-01-01T00:00:00Z",
					UpdatedAt: "2024-01-01T00:00:00Z",
				},
			},
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
			apps: []appItem{
				{
					AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
					AppName:   "solo-app",
					Status:    api.ListAppsResponseBodyAppsStatusACTIVE,
					CreatedAt: "2024-01-01T00:00:00Z",
					UpdatedAt: "2024-01-01T00:00:00Z",
				},
			},
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
			apps:     []appItem{},
			expected: []string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			// Convert test data to actual type
			apps := make([]struct {
				AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
				AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
				AppId         uuid.UUID                              `json:"appId"`
				AppName       string                                 `json:"appName"`
				CreatedAt     string                                 `json:"createdAt"`
				CustomAuth    *struct {
					Audience string `json:"audience"`
					Issuer   string `json:"issuer"`
				} `json:"customAuth,omitempty"`
				OrgId         *string                        `json:"orgId,omitempty"`
				ProviderError *string                        `json:"providerError,omitempty"`
				ProviderUrl   *string                        `json:"providerUrl,omitempty"`
				Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
				UpdatedAt     string                         `json:"updatedAt"`
			}, len(tt.apps))

			for i, app := range tt.apps {
				apps[i].AppId = app.AppId
				apps[i].AppName = app.AppName
				apps[i].Status = app.Status
				apps[i].ProviderUrl = app.ProviderUrl
				apps[i].CreatedAt = app.CreatedAt
				apps[i].UpdatedAt = app.UpdatedAt
			}

			dw := NewDataWriter(&buf, "table")
			err := displayAppsTable(apps, false, dw) // not verbose
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

func TestDisplayAppsJSON(t *testing.T) {
	apps := []appItem{
		{
			AppId:       uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
			AppName:     "test-app",
			Status:      api.ListAppsResponseBodyAppsStatusACTIVE,
			ProviderUrl: ptr("https://example.com"),
			CreatedAt:   "2024-01-01T00:00:00Z",
			UpdatedAt:   "2024-01-01T00:00:00Z",
		},
		{
			AppId:     uuid.MustParse("223e4567-e89b-12d3-a456-426614174000"),
			AppName:   "another-app",
			Status:    api.ListAppsResponseBodyAppsStatusPENDING,
			CreatedAt: "2024-01-02T00:00:00Z",
			UpdatedAt: "2024-01-02T00:00:00Z",
		},
	}

	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	// Convert test data
	actualApps := make([]struct {
		AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
		AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
		AppId         uuid.UUID                              `json:"appId"`
		AppName       string                                 `json:"appName"`
		CreatedAt     string                                 `json:"createdAt"`
		CustomAuth    *struct {
			Audience string `json:"audience"`
			Issuer   string `json:"issuer"`
		} `json:"customAuth,omitempty"`
		OrgId         *string                        `json:"orgId,omitempty"`
		ProviderError *string                        `json:"providerError,omitempty"`
		ProviderUrl   *string                        `json:"providerUrl,omitempty"`
		Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
		UpdatedAt     string                         `json:"updatedAt"`
	}, len(apps))

	for i, app := range apps {
		actualApps[i].AppId = app.AppId
		actualApps[i].AppName = app.AppName
		actualApps[i].Status = app.Status
		actualApps[i].ProviderUrl = app.ProviderUrl
		actualApps[i].CreatedAt = app.CreatedAt
		actualApps[i].UpdatedAt = app.UpdatedAt
	}

	dw := NewDataWriter(&buf, "json")
	err := dw.WriteStruct(actualApps)
	require.NoError(t, err)

	// Verify JSON output
	var result []map[string]interface{}
	err = json.Unmarshal(buf.Bytes(), &result)
	require.NoError(t, err)
	assert.Len(t, result, 2)
	assert.Equal(t, "test-app", result[0]["appName"])
	assert.Equal(t, "another-app", result[1]["appName"])
	assert.Equal(t, "ACTIVE", result[0]["status"])
	assert.Equal(t, "PENDING", result[1]["status"])
}

func TestListCommandColorCoding(t *testing.T) {
	// Test that different statuses get different colors (when color is enabled)
	statuses := []struct {
		status   api.ListAppsResponseBodyAppsStatus
		contains string
	}{
		{api.ListAppsResponseBodyAppsStatusACTIVE, "ACTIVE"},
		{api.ListAppsResponseBodyAppsStatusFAILED, "FAILED"},
		{api.ListAppsResponseBodyAppsStatusPENDING, "PENDING"},
		{api.ListAppsResponseBodyAppsStatusCREATING, "CREATING"},
		{api.ListAppsResponseBodyAppsStatusDELETING, "DELETING"},
		{api.ListAppsResponseBodyAppsStatusDELETED, "DELETED"},
	}

	for _, st := range statuses {
		t.Run(string(st.status), func(t *testing.T) {
			var buf bytes.Buffer
			oldOutput := colorOutput
			colorOutput = &buf
			defer func() { colorOutput = oldOutput }()

			apps := []struct {
				AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
				AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
				AppId         uuid.UUID                              `json:"appId"`
				AppName       string                                 `json:"appName"`
				CreatedAt     string                                 `json:"createdAt"`
				CustomAuth    *struct {
					Audience string `json:"audience"`
					Issuer   string `json:"issuer"`
				} `json:"customAuth,omitempty"`
				OrgId         *string                        `json:"orgId,omitempty"`
				ProviderError *string                        `json:"providerError,omitempty"`
				ProviderUrl   *string                        `json:"providerUrl,omitempty"`
				Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
				UpdatedAt     string                         `json:"updatedAt"`
			}{
				{
					AppId:     uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
					AppName:   "test-app",
					Status:    st.status,
					CreatedAt: "2024-01-01T00:00:00Z",
					UpdatedAt: "2024-01-01T00:00:00Z",
				},
			}

			dw := NewDataWriter(&buf, "table")
			err := displayAppsTable(apps, false, dw) // not verbose
			require.NoError(t, err)
			assert.Contains(t, buf.String(), st.contains)
		})
	}
}

func TestListEmptyApps(t *testing.T) {
	// Test handling of empty app list
	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	apps := []struct {
		AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
		AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
		AppId         uuid.UUID                              `json:"appId"`
		AppName       string                                 `json:"appName"`
		CreatedAt     string                                 `json:"createdAt"`
		CustomAuth    *struct {
			Audience string `json:"audience"`
			Issuer   string `json:"issuer"`
		} `json:"customAuth,omitempty"`
		OrgId         *string                        `json:"orgId,omitempty"`
		ProviderError *string                        `json:"providerError,omitempty"`
		ProviderUrl   *string                        `json:"providerUrl,omitempty"`
		Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
		UpdatedAt     string                         `json:"updatedAt"`
	}{}

	dw := NewDataWriter(&buf, "table")
	err := displayAppsTable(apps, false, dw)
	require.NoError(t, err)

	// Should still show header and total
	output := buf.String()
	assert.Contains(t, output, "NAME")
	assert.Contains(t, output, "Total: 0 applications")
}

func TestListWithURLs(t *testing.T) {
	var buf bytes.Buffer
	oldOutput := colorOutput
	colorOutput = &buf
	defer func() { colorOutput = oldOutput }()

	apps := []struct {
		AccessControl *api.ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
		AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
		AppId         uuid.UUID                              `json:"appId"`
		AppName       string                                 `json:"appName"`
		CreatedAt     string                                 `json:"createdAt"`
		CustomAuth    *struct {
			Audience string `json:"audience"`
			Issuer   string `json:"issuer"`
		} `json:"customAuth,omitempty"`
		OrgId         *string                        `json:"orgId,omitempty"`
		ProviderError *string                        `json:"providerError,omitempty"`
		ProviderUrl   *string                        `json:"providerUrl,omitempty"`
		Status        api.ListAppsResponseBodyAppsStatus `json:"status"`
		UpdatedAt     string                         `json:"updatedAt"`
	}{
		{
			AppId:       uuid.MustParse("123e4567-e89b-12d3-a456-426614174000"),
			AppName:     "with-url",
			Status:      api.ListAppsResponseBodyAppsStatusACTIVE,
			ProviderUrl: ptr("https://example.com"),
			CreatedAt:   "2024-01-01T00:00:00Z",
			UpdatedAt:   "2024-01-01T00:00:00Z",
		},
		{
			AppId:     uuid.MustParse("223e4567-e89b-12d3-a456-426614174000"),
			AppName:   "without-url",
			Status:    api.ListAppsResponseBodyAppsStatusACTIVE,
			CreatedAt: "2024-01-01T00:00:00Z",
			UpdatedAt: "2024-01-01T00:00:00Z",
		},
	}

	dw := NewDataWriter(&buf, "table")
	err := displayAppsTable(apps, false, dw)
	require.NoError(t, err)

	output := buf.String()
	assert.Contains(t, output, "https://example.com")
	// App without URL should show "-"
	lines := strings.Split(output, "\n")
	for _, line := range lines {
		if strings.Contains(line, "without-url") {
			assert.Contains(t, line, "-")
			break
		}
	}
}

// Test helper types
type appItem struct {
	AppId         uuid.UUID
	AppName       string
	Status        api.ListAppsResponseBodyAppsStatus
	ProviderUrl   *string
	ProviderError *string
	CreatedAt     string
	UpdatedAt     string
}

func ptr(s string) *string {
	return &s
}
