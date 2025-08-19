package api

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/google/uuid"
	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/fastertools/ftl-cli/internal/auth"
)

const (
	// DefaultAPIBaseURL is the default FTL API endpoint
	DefaultAPIBaseURL = "https://vnwyancgjj.execute-api.us-west-2.amazonaws.com"
)

// FTLClient wraps the generated API client with authentication
type FTLClient struct {
	client      *ClientWithResponses
	authManager *auth.Manager
	baseURL     string
}

// NewFTLClient creates a new FTL API client with authentication
func NewFTLClient(authManager *auth.Manager, baseURL string) (*FTLClient, error) {
	if baseURL == "" {
		baseURL = DefaultAPIBaseURL
	}

	// Create HTTP client with auth interceptor
	httpClient := &authHTTPClient{
		authManager: authManager,
		underlying: &http.Client{
			Timeout: 30 * time.Second,
		},
	}

	// Create the generated client
	client, err := NewClientWithResponses(baseURL, WithHTTPClient(httpClient))
	if err != nil {
		return nil, fmt.Errorf("failed to create API client: %w", err)
	}

	return &FTLClient{
		client:      client,
		authManager: authManager,
		baseURL:     baseURL,
	}, nil
}

// authHTTPClient adds authentication headers to requests
type authHTTPClient struct {
	authManager *auth.Manager
	underlying  *http.Client
}

// Do implements the HTTP client interface with authentication
func (c *authHTTPClient) Do(req *http.Request) (*http.Response, error) {
	// Get the auth token
	token, err := c.authManager.GetToken(req.Context())
	if err != nil {
		return nil, fmt.Errorf("failed to get auth token: %w", err)
	}

	// Add authorization header
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", token))

	// Execute the request
	return c.underlying.Do(req)
}

// Client returns the underlying generated client for direct access
func (c *FTLClient) Client() *ClientWithResponses {
	return c.client
}

// GetBaseURL returns the base URL for the API
func (c *FTLClient) GetBaseURL() string {
	return c.baseURL
}

// GetAuthToken returns the current auth token
func (c *FTLClient) GetAuthToken(ctx context.Context) (string, error) {
	return c.authManager.GetToken(ctx)
}

// parseUUID converts a string to an openapi UUID type
func parseUUID(s string) (openapi_types.UUID, error) {
	u, err := uuid.Parse(s)
	if err != nil {
		return openapi_types.UUID{}, err
	}
	return openapi_types.UUID(u), nil
}

// Apps API methods

// ListApps retrieves a list of applications
func (c *FTLClient) ListApps(ctx context.Context, params *ListAppsParams) (*ListAppsResponseBody, error) {
	resp, err := c.client.ListAppsWithResponse(ctx, params)
	if err != nil {
		return nil, fmt.Errorf("failed to list apps: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON200 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON200, nil
}

// CreateApp creates a new application
func (c *FTLClient) CreateApp(ctx context.Context, request CreateAppRequest) (*CreateAppResponseBody, error) {
	params := &CreateAppParams{}
	resp, err := c.client.CreateAppWithResponse(ctx, params, request)
	if err != nil {
		return nil, fmt.Errorf("failed to create app: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusCreated {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON201 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON201, nil
}

// GetApp retrieves details of a specific app
func (c *FTLClient) GetApp(ctx context.Context, appID string) (*App, error) {
	appUUID, err := parseUUID(appID)
	if err != nil {
		return nil, fmt.Errorf("invalid app ID: %w", err)
	}
	params := &GetAppParams{}
	resp, err := c.client.GetAppWithResponse(ctx, appUUID, params)
	if err != nil {
		return nil, fmt.Errorf("failed to get app: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON200 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON200, nil
}

// DeleteApp deletes an application
func (c *FTLClient) DeleteApp(ctx context.Context, appID string) error {
	appUUID, err := parseUUID(appID)
	if err != nil {
		return fmt.Errorf("invalid app ID: %w", err)
	}
	params := &DeleteAppParams{}
	resp, err := c.client.DeleteAppWithResponse(ctx, appUUID, params)
	if err != nil {
		return fmt.Errorf("failed to delete app: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusAccepted && resp.HTTPResponse.StatusCode != http.StatusNoContent {
		return fmt.Errorf("API error: %s", string(resp.Body))
	}

	return nil
}

// Registry API methods

// CreateECRToken creates a temporary ECR authorization token
func (c *FTLClient) CreateECRToken(ctx context.Context, appID string, components []string) (*CreateEcrTokenResponseBody, error) {
	appUUID, err := parseUUID(appID)
	if err != nil {
		return nil, fmt.Errorf("invalid app ID: %w", err)
	}
	request := CreateEcrTokenRequest{
		AppId:      appUUID,
		Components: components,
	}
	params := &CreateEcrTokenParams{}

	resp, err := c.client.CreateEcrTokenWithResponse(ctx, params, request)
	if err != nil {
		return nil, fmt.Errorf("failed to create ECR token: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON200 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON200, nil
}

// Component API methods

// UpdateComponents updates the component list for an app
func (c *FTLClient) UpdateComponents(ctx context.Context, appID string, request UpdateComponentsRequest) (*UpdateComponentsResponseBody, error) {
	appUUID, err := parseUUID(appID)
	if err != nil {
		return nil, fmt.Errorf("invalid app ID: %w", err)
	}
	params := &UpdateComponentsParams{}

	resp, err := c.client.UpdateComponentsWithResponse(ctx, appUUID, params, request)
	if err != nil {
		return nil, fmt.Errorf("failed to update components: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON200 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON200, nil
}

// Deployment API methods

// CreateDeployment triggers a new deployment for an application
func (c *FTLClient) CreateDeployment(ctx context.Context, appID string, request CreateDeploymentRequest) (*CreateDeploymentResponseBody, error) {
	appUUID, err := parseUUID(appID)
	if err != nil {
		return nil, fmt.Errorf("invalid app ID: %w", err)
	}
	params := &CreateDeploymentParams{}

	resp, err := c.client.CreateDeploymentWithResponse(ctx, appUUID, params, request)
	if err != nil {
		return nil, fmt.Errorf("failed to create deployment: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusAccepted {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON202 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON202, nil
}

// Organization API methods

// GetUserOrgs retrieves the organizations for the authenticated user
func (c *FTLClient) GetUserOrgs(ctx context.Context) (*GetUserOrgsResponseBody, error) {
	params := &GetUserOrgsParams{}
	resp, err := c.client.GetUserOrgsWithResponse(ctx, params)
	if err != nil {
		return nil, fmt.Errorf("failed to get user orgs: %w", err)
	}

	if resp.HTTPResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API error: %s", string(resp.Body))
	}

	if resp.JSON200 == nil {
		return nil, fmt.Errorf("unexpected response format")
	}

	return resp.JSON200, nil
}
