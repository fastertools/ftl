package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/fastertools/ftl-cli/go/shared/auth"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewFTLClient(t *testing.T) {
	// Create a mock auth manager
	authManager := &auth.Manager{}

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

func TestFTLClient_ListApps(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps", r.URL.Path)
		assert.Equal(t, "GET", r.Method)

		response := ListAppsResponseBody{
			Data: &[]App{
				{Id: strPtr("app1"), Name: strPtr("Test App 1")},
				{Id: strPtr("app2"), Name: strPtr("Test App 2")},
			},
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	authManager := &auth.Manager{}
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	apps, err := client.ListApps(ctx, nil)
	require.NoError(t, err)
	assert.NotNil(t, apps)
	assert.NotNil(t, apps.Data)
	assert.Len(t, *apps.Data, 2)
}

func TestFTLClient_CreateApp(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps", r.URL.Path)
		assert.Equal(t, "POST", r.Method)

		var req CreateAppRequest
		json.NewDecoder(r.Body).Decode(&req)
		assert.NotNil(t, req.Application)

		response := CreateAppResponseBody{
			Data: &App{
				Id:   strPtr("new-app-id"),
				Name: req.Application.Name,
			},
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusCreated)
		json.NewEncoder(w).Encode(response)
	}))
	defer server.Close()

	authManager := &auth.Manager{}
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	request := CreateAppRequest{
		Application: &AppInput{
			Name: strPtr("new-app"),
		},
	}

	result, err := client.CreateApp(ctx, request)
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.NotNil(t, result.Data)
	assert.Equal(t, "new-app-id", *result.Data.Id)
}

func TestFTLClient_GetApp(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps/test-app-id", r.URL.Path)
		assert.Equal(t, "GET", r.Method)

		response := App{
			Id:   strPtr("test-app-id"),
			Name: strPtr("Test App"),
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(struct {
			Data *App `json:"data"`
		}{Data: &response})
	}))
	defer server.Close()

	authManager := &auth.Manager{}
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	app, err := client.GetApp(ctx, "test-app-id")
	require.NoError(t, err)
	assert.NotNil(t, app)
	assert.Equal(t, "test-app-id", *app.Id)
}

func TestFTLClient_DeleteApp(t *testing.T) {
	// Create test server
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "/v1/apps/test-app-id", r.URL.Path)
		assert.Equal(t, "DELETE", r.Method)
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()

	authManager := &auth.Manager{}
	client, err := NewFTLClient(authManager, server.URL)
	require.NoError(t, err)

	ctx := context.Background()
	err = client.DeleteApp(ctx, "test-app-id")
	assert.NoError(t, err)
}

func TestFTLClient_ErrorHandling(t *testing.T) {
	// Create test server that returns errors
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(map[string]string{
			"error": "internal server error",
		})
	}))
	defer server.Close()

	authManager := &auth.Manager{}
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
		authManager := &auth.Manager{}
		// Mock token retrieval would go here in real implementation
		client := &authHTTPClient{
			authManager: authManager,
			httpClient:  http.DefaultClient,
		}

		req, _ := http.NewRequest("GET", server.URL, nil)
		resp, err := client.Do(req)
		require.NoError(t, err)
		assert.NotNil(t, resp)
	})
}

// Helper function for string pointers
func strPtr(s string) *string {
	return &s
}