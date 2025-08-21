package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/fastertools/ftl/internal/auth"
	"github.com/google/uuid"
	openapi_types "github.com/oapi-codegen/runtime/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewFTLClient(t *testing.T) {
	// Create a mock auth manager
	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)

	tests := []struct {
		name    string
		baseURL string
		wantErr bool
	}{
		{
			name:    "valid URL",
			baseURL: "https://api.example.com",
			wantErr: false,
		},
		{
			name:    "empty URL uses default",
			baseURL: "",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			client, err := NewFTLClient(authManager, tt.baseURL)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				require.NoError(t, err)
				assert.NotNil(t, client)
				assert.NotNil(t, client.Client())
			}
		})
	}
}

// mockCredentialStore is a simple mock for testing
type mockCredentialStore struct {
	creds *auth.Credentials
}

func (m *mockCredentialStore) Load() (*auth.Credentials, error) {
	if m.creds == nil {
		return nil, fmt.Errorf("not logged in")
	}
	return m.creds, nil
}

func (m *mockCredentialStore) Save(creds *auth.Credentials) error {
	m.creds = creds
	return nil
}

func (m *mockCredentialStore) Delete() error {
	m.creds = nil
	return nil
}

func (m *mockCredentialStore) Exists() bool {
	return m.creds != nil
}

// M2M methods
func (m *mockCredentialStore) StoreToken(token string, expiresIn int) error {
	return nil
}

func (m *mockCredentialStore) GetM2MConfig() (*auth.M2MConfig, error) {
	return nil, fmt.Errorf("no M2M config")
}

func (m *mockCredentialStore) StoreM2MConfig(config *auth.M2MConfig) error {
	return nil
}

func (m *mockCredentialStore) SetActorType(actorType string) error {
	return nil
}

func (m *mockCredentialStore) GetActorType() (string, error) {
	return "user", nil
}

func TestFTLClient_ListApps(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps", r.URL.Path)
		assert.Equal(t, "GET", r.Method)

		// The actual API returns apps directly in an array
		response := ListAppsResponseBody{
			Apps: []struct {
				AccessControl *ListAppsResponseBodyAppsAccessControl `json:"accessControl,omitempty"`
				AllowedRoles  *[]string                              `json:"allowedRoles,omitempty"`
				AppId         openapi_types.UUID                     `json:"appId"`
				AppName       string                                 `json:"appName"`
				CreatedAt     string                                 `json:"createdAt"`
				CustomAuth    *struct {
					Audience string `json:"audience"`
					Issuer   string `json:"issuer"`
				} `json:"customAuth,omitempty"`
				LatestDeployment *struct {
					CreatedAt          *float32                                       `json:"createdAt,omitempty"`
					DeployedAt         *float32                                       `json:"deployedAt,omitempty"`
					DeploymentDuration *float32                                       `json:"deploymentDuration,omitempty"`
					DeploymentId       string                                         `json:"deploymentId"`
					Environment        *string                                        `json:"environment,omitempty"`
					Status             ListAppsResponseBodyAppsLatestDeploymentStatus `json:"status"`
					StatusMessage      *string                                        `json:"statusMessage,omitempty"`
				} `json:"latestDeployment"`
				OrgId         *string                        `json:"orgId,omitempty"`
				ProviderError *string                        `json:"providerError,omitempty"`
				ProviderUrl   *string                        `json:"providerUrl,omitempty"`
				Status        ListAppsResponseBodyAppsStatus `json:"status"`
				UpdatedAt     string                         `json:"updatedAt"`
			}{
				{AppId: openapi_types.UUID(uuid.New()), AppName: "Test App 1", CreatedAt: "2024-01-01", UpdatedAt: "2024-01-01", Status: ACTIVE},
				{AppId: openapi_types.UUID(uuid.New()), AppName: "Test App 2", CreatedAt: "2024-01-01", UpdatedAt: "2024-01-01", Status: ACTIVE},
			},
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_ = json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	apps, err := client.ListApps(ctx, nil)
	require.NoError(t, err)
	assert.NotNil(t, apps)
	assert.Len(t, apps.Apps, 2)
}

func TestFTLClient_CreateApp(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps", r.URL.Path)
		assert.Equal(t, "POST", r.Method)

		var req CreateAppRequest
		_ = json.NewDecoder(r.Body).Decode(&req)
		assert.NotEmpty(t, req.AppName)

		response := CreateAppResponseBody{
			AppId:     openapi_types.UUID(uuid.New()),
			AppName:   req.AppName,
			CreatedAt: "2024-01-01",
			UpdatedAt: "2024-01-01",
			Status:    CreateAppResponseBodyStatusPENDING,
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		_ = json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	request := CreateAppRequest{
		AppName: "new-app",
	}

	result, err := client.CreateApp(ctx, request)
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.NotEmpty(t, result.AppId)
	assert.Equal(t, "new-app", result.AppName)
}

func TestFTLClient_GetApp(t *testing.T) {
	testID := uuid.New().String()
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, fmt.Sprintf("/v1/apps/%s", testID), r.URL.Path)
		assert.Equal(t, "GET", r.Method)

		response := App{
			AppId:     openapi_types.UUID(uuid.New()),
			AppName:   "Test App",
			CreatedAt: "2024-01-01",
			UpdatedAt: "2024-01-01",
			Status:    AppStatusACTIVE,
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_ = json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	app, err := client.GetApp(ctx, testID)
	require.NoError(t, err)
	assert.NotNil(t, app)
	assert.NotEmpty(t, app.AppId)
}

func TestFTLClient_DeleteApp(t *testing.T) {
	testID := uuid.New().String()
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, fmt.Sprintf("/v1/apps/%s", testID), r.URL.Path)
		assert.Equal(t, "DELETE", r.Method)
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	err = client.DeleteApp(ctx, testID)
	assert.NoError(t, err)
}

func TestFTLClient_ErrorHandling(t *testing.T) {
	// Create test server that returns errors
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		_ = json.NewEncoder(w).Encode(map[string]string{
			"error": "internal server error",
		})
	}))
	defer server.Close()

	// Create auth manager with mock store
	mockStore := &mockCredentialStore{
		creds: &auth.Credentials{
			AccessToken: "test-token",
			ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
		},
	}
	authManager := auth.NewManager(mockStore, nil)
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()

	t.Run("ListApps error", func(t *testing.T) {
		_, err := client.ListApps(ctx, nil)
		assert.Error(t, err)
	})

	t.Run("GetApp error", func(t *testing.T) {
		_, err := client.GetApp(ctx, "test-app")
		assert.Error(t, err)
	})

	t.Run("DeleteApp error", func(t *testing.T) {
		err := client.DeleteApp(ctx, "test-app")
		assert.Error(t, err)
	})
}

func TestAuthHTTPClient(t *testing.T) {
	// Create test server that checks for auth header
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		if authHeader == "" {
			w.WriteHeader(http.StatusUnauthorized)
		} else {
			w.WriteHeader(http.StatusOK)
		}
	}))
	defer server.Close()

	t.Run("with auth token", func(t *testing.T) {
		// Create auth manager with mock store
		mockStore := &mockCredentialStore{
			creds: &auth.Credentials{
				AccessToken: "test-token",
				ExpiresAt:   timePtr(time.Now().Add(time.Hour)),
			},
		}
		authManager := auth.NewManager(mockStore, nil)
		// Mock token retrieval would go here in real implementation
		client := &authHTTPClient{
			authManager: authManager,
			underlying:  http.DefaultClient,
		}

		req, _ := http.NewRequest("GET", server.URL, nil)
		resp, err := client.Do(req)
		require.NoError(t, err)
		assert.NotNil(t, resp)
	})
}

// Helper function for time pointers
func timePtr(t time.Time) *time.Time {
	return &t
}
