package ftl

import (
	"encoding/json"
	"fmt"
)

// DeploymentRequest represents a deployment request to the FTL platform
// This is the contract between CLI and platform backend
type DeploymentRequest struct {
	// The FTL application configuration
	Application *Application `json:"application"`
	
	// Environment-specific variables
	Variables map[string]string `json:"variables,omitempty"`
	
	// Environment name (e.g., production, staging)
	Environment string `json:"environment,omitempty"`
	
	// Access control override (if different from app config)
	AccessControl *AccessMode `json:"access_control,omitempty"`
	
	// Custom auth configuration (for custom access mode)
	CustomAuth *CustomAuthConfig `json:"custom_auth,omitempty"`
	
	// Allowed roles for org access mode
	AllowedRoles []string `json:"allowed_roles,omitempty"`
}

// CustomAuthConfig represents custom authentication configuration
type CustomAuthConfig struct {
	Issuer   string   `json:"issuer"`
	Audience []string `json:"audience"`
}

// DeploymentResponse represents the response from a deployment request
type DeploymentResponse struct {
	DeploymentID string `json:"deployment_id"`
	AppID        string `json:"app_id"`
	AppName      string `json:"app_name"`
	Status       string `json:"status"`
	Message      string `json:"message,omitempty"`
}

// ComponentStatus represents the status of a component in the deployment
type ComponentStatus struct {
	ID       string `json:"id"`
	Status   string `json:"status"`
	Registry string `json:"registry,omitempty"`
	Package  string `json:"package,omitempty"`
	Version  string `json:"version,omitempty"`
	Error    string `json:"error,omitempty"`
}

// DeploymentStatus represents the overall deployment status
type DeploymentStatus struct {
	DeploymentID string            `json:"deployment_id"`
	Status       string            `json:"status"`
	ProviderURL  string            `json:"provider_url,omitempty"`
	Components   []ComponentStatus `json:"components"`
	Error        string            `json:"error,omitempty"`
	UpdatedAt    string            `json:"updated_at"`
}

// PrepareDeployment prepares a deployment request from an FTL application
func PrepareDeployment(app *Application, opts DeploymentOptions) (*DeploymentRequest, error) {
	// Validate the application
	if err := app.Validate(); err != nil {
		return nil, fmt.Errorf("invalid application: %w", err)
	}
	
	// Ensure all local components have been pushed to registry
	for _, comp := range app.Components {
		if comp.Source.IsLocal() {
			return nil, fmt.Errorf("component %s has local source, must be pushed to registry first", comp.ID)
		}
	}
	
	req := &DeploymentRequest{
		Application: app,
		Variables:   opts.Variables,
		Environment: opts.Environment,
	}
	
	// Apply access control override if specified
	if opts.AccessControl != "" {
		mode := AccessMode(opts.AccessControl)
		req.AccessControl = &mode
	}
	
	// Add custom auth if needed
	if app.Access == AccessCustom || (req.AccessControl != nil && *req.AccessControl == AccessCustom) {
		if app.Auth.Provider == AuthProviderCustom {
			req.CustomAuth = &CustomAuthConfig{
				Issuer:   app.Auth.JWTIssuer,
				Audience: []string{app.Auth.JWTAudience},
			}
		}
	}
	
	// Add allowed roles for org mode
	if (app.Access == AccessOrg || (req.AccessControl != nil && *req.AccessControl == AccessOrg)) && len(opts.AllowedRoles) > 0 {
		req.AllowedRoles = opts.AllowedRoles
	}
	
	return req, nil
}

// DeploymentOptions contains options for deployment
type DeploymentOptions struct {
	Environment   string
	Variables     map[string]string
	AccessControl string
	AllowedRoles  []string
}

// ProcessDeploymentRequest processes a deployment request on the platform side
// This is what the backend Lambda would use
func ProcessDeploymentRequest(req *DeploymentRequest) (*SpinManifest, error) {
	// Apply any overrides from the deployment request
	app := req.Application
	
	if req.AccessControl != nil {
		app.Access = *req.AccessControl
	}
	
	// Set defaults
	app.SetDefaults()
	
	// Validate
	if err := app.Validate(); err != nil {
		return nil, fmt.Errorf("invalid application in deployment request: %w", err)
	}
	
	// Synthesize the Spin manifest
	synth := NewSynthesizer()
	manifest, err := synth.SynthesizeToSpin(app)
	if err != nil {
		return nil, fmt.Errorf("failed to synthesize manifest: %w", err)
	}
	
	// Apply deployment variables to the manifest
	if len(req.Variables) > 0 {
		if manifest.Variables == nil {
			manifest.Variables = make(map[string]SpinVariable)
		}
		
		for key, value := range req.Variables {
			manifest.Variables[key] = SpinVariable{
				Default: value,
			}
		}
	}
	
	return manifest, nil
}

// MarshalJSON implements custom JSON marshalling for DeploymentRequest
func (d *DeploymentRequest) MarshalJSON() ([]byte, error) {
	// We want to ensure the Application is properly serialized
	type Alias DeploymentRequest
	return json.Marshal(&struct {
		*Alias
	}{
		Alias: (*Alias)(d),
	})
}

// UnmarshalJSON implements custom JSON unmarshalling for DeploymentRequest
func (d *DeploymentRequest) UnmarshalJSON(data []byte) error {
	type Alias DeploymentRequest
	aux := &struct {
		*Alias
	}{
		Alias: (*Alias)(d),
	}
	
	if err := json.Unmarshal(data, &aux); err != nil {
		return err
	}
	
	return nil
}