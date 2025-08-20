package deploy

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/fastertools/ftl-cli/internal/api"
	"github.com/stretchr/testify/assert"
)

func TestExtractRegion(t *testing.T) {
	tests := []struct {
		name           string
		functionURL    string
		registryURI    string
		expectedRegion string
	}{
		{
			name:           "Extract from Lambda function URL",
			functionURL:    "https://abc123.lambda-url.us-west-2.on.aws/",
			registryURI:    "",
			expectedRegion: "us-west-2",
		},
		{
			name:           "Extract from ECR registry URI",
			functionURL:    "",
			registryURI:    "795394005211.dkr.ecr.us-east-1.amazonaws.com",
			expectedRegion: "us-east-1",
		},
		{
			name:           "Lambda URL takes precedence",
			functionURL:    "https://xyz789.lambda-url.eu-west-1.on.aws/",
			registryURI:    "795394005211.dkr.ecr.us-east-1.amazonaws.com",
			expectedRegion: "eu-west-1",
		},
		{
			name:           "Default to us-west-2 when can't extract",
			functionURL:    "https://some-invalid-url.com",
			registryURI:    "invalid-registry",
			expectedRegion: "us-west-2",
		},
		{
			name:           "Handle different Lambda URL format",
			functionURL:    "https://def456.lambda-url.ap-southeast-2.on.aws/deploy",
			registryURI:    "",
			expectedRegion: "ap-southeast-2",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			region := extractRegion(tt.functionURL, tt.registryURI)
			assert.Equal(t, tt.expectedRegion, region)
		})
	}
}

func TestStreamingDeploySuccess(t *testing.T) {
	// Create a test server that simulates streaming NDJSON responses
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Verify request headers
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))
		assert.Equal(t, "application/x-ndjson", r.Header.Get("Accept"))

		// Verify AWS signature headers are present
		assert.NotEmpty(t, r.Header.Get("Authorization"))
		assert.Contains(t, r.Header.Get("Authorization"), "AWS4-HMAC-SHA256")

		// Send streaming NDJSON response
		w.Header().Set("Content-Type", "application/x-ndjson")
		w.WriteHeader(http.StatusOK)

		events := []StreamEvent{
			{Type: "progress", Message: "Starting deployment", Timestamp: time.Now().Unix()},
			{Type: "progress", Message: "Building application", Timestamp: time.Now().Unix()},
			{Type: "progress", Message: "Deploying to platform", Timestamp: time.Now().Unix()},
			{Type: "complete", Message: "Deployment successful", DeploymentID: "deploy-123", URL: "https://app.example.com", Timestamp: time.Now().Unix()},
		}

		encoder := json.NewEncoder(w)
		for _, event := range events {
			_ = encoder.Encode(event)
			w.(http.Flusher).Flush()
			time.Sleep(10 * time.Millisecond) // Simulate processing time
		}
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
			Region:      "us-west-2",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app", "version": "1.0.0"}`)

	var receivedEvents []StreamEvent
	err := deployer.Deploy(context.Background(), ftlConfig, creds, "", func(event StreamEvent) {
		receivedEvents = append(receivedEvents, event)
	})

	assert.NoError(t, err)
	assert.Len(t, receivedEvents, 4)
	assert.Equal(t, "complete", receivedEvents[3].Type)
	assert.Equal(t, "deploy-123", receivedEvents[3].DeploymentID)
}

func TestStreamingDeployError(t *testing.T) {
	// Create a test server that returns an error event
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/x-ndjson")
		w.WriteHeader(http.StatusOK)

		events := []StreamEvent{
			{Type: "progress", Message: "Starting deployment", Timestamp: time.Now().Unix()},
			{Type: "error", Message: "Failed to build application: syntax error", Timestamp: time.Now().Unix()},
		}

		encoder := json.NewEncoder(w)
		for _, event := range events {
			_ = encoder.Encode(event)
			w.(http.Flusher).Flush()
		}
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
			Region:      "us-west-2",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app", "version": "1.0.0"}`)

	err := deployer.Deploy(context.Background(), ftlConfig, creds, "", nil)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "Failed to build application: syntax error")
}

func TestStreamingDeployHTTPError(t *testing.T) {
	// Create a test server that returns a 403 error
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusForbidden)
		fmt.Fprintf(w, `{"message": "The request signature we calculated does not match"}`)
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
			Region:      "us-west-2",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app", "version": "1.0.0"}`)

	err := deployer.Deploy(context.Background(), ftlConfig, creds, "", nil)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "deployment failed with status 403")
}

func TestStreamingDeployWithEnvironment(t *testing.T) {
	// Create a test server that verifies the environment parameter
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Verify environment parameter
		assert.Equal(t, "staging", r.URL.Query().Get("environment"))

		w.Header().Set("Content-Type", "application/x-ndjson")
		w.WriteHeader(http.StatusOK)

		event := StreamEvent{Type: "complete", Message: "Success", Timestamp: time.Now().Unix()}
		_ = json.NewEncoder(w).Encode(event)
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
			Region:      "us-west-2",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment with environment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app", "version": "1.0.0"}`)

	err := deployer.Deploy(context.Background(), ftlConfig, creds, "staging", nil)

	assert.NoError(t, err)
}

func TestStreamingDeployMalformedJSON(t *testing.T) {
	// Create a test server that sends malformed JSON
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/x-ndjson")
		w.WriteHeader(http.StatusOK)

		// Send valid event
		fmt.Fprintln(w, `{"type":"progress","message":"Starting"}`)
		w.(http.Flusher).Flush()

		// Send malformed JSON (should be logged but not fail)
		fmt.Fprintln(w, `{malformed json}`)
		w.(http.Flusher).Flush()

		// Send completion event
		fmt.Fprintln(w, `{"type":"complete","message":"Done"}`)
		w.(http.Flusher).Flush()
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app"}`)

	var eventCount int
	err := deployer.Deploy(context.Background(), ftlConfig, creds, "", func(event StreamEvent) {
		eventCount++
	})

	// Should succeed despite malformed JSON
	assert.NoError(t, err)
	assert.Equal(t, 2, eventCount) // Only valid events counted
}

func TestStreamingDeployIncompleteStream(t *testing.T) {
	// Create a test server that closes connection without completion
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/x-ndjson")
		w.WriteHeader(http.StatusOK)

		// Send progress but no completion
		fmt.Fprintln(w, `{"type":"progress","message":"Starting deployment"}`)
		w.(http.Flusher).Flush()

		// Abruptly close connection
	}))
	defer server.Close()

	// Create mock credentials
	creds := &api.CreateDeployCredentialsResponse{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: "795394005211.dkr.ecr.us-west-2.amazonaws.com",
		},
		Deployment: struct {
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: server.URL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		},
	}

	// Create deployer and test deployment
	deployer := NewStreamingDeployer()
	ftlConfig := []byte(`{"name": "test-app"}`)

	err := deployer.Deploy(context.Background(), ftlConfig, creds, "", nil)

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "deployment stream ended without completion")
}
