// Package config defines the FTL configuration schema
// This is used both locally and for deployments
//
// NOTE: This package includes fields that are NOT exposed to end users:
// - KeyValueStores, SQLiteDatabases, AIModels in ComponentConfig
// These fields exist for:
// 1. Future platform features (not yet implemented)
// 2. API compatibility with potential backend services
// 3. Internal platform use only
//
// User-facing configuration is defined in the ftl package types,
// which intentionally excludes these fields to prevent misuse.
// The CUE synthesis patterns only copy allowed fields from user config.
package config

import (
	"fmt"
	"time"
)

// FTLConfig represents the top-level FTL configuration
// This is the source of truth for both local development and platform deployments
type FTLConfig struct {
	Application ApplicationConfig        `yaml:"application" json:"application"`
	Components  []ComponentConfig        `yaml:"components,omitempty" json:"components,omitempty"`
	Triggers    []TriggerConfig          `yaml:"triggers,omitempty" json:"triggers,omitempty"`
	Variables   map[string]string        `yaml:"variables,omitempty" json:"variables,omitempty"`
	MCP         *MCPConfig               `yaml:"mcp,omitempty" json:"mcp,omitempty"`
}

// ApplicationConfig defines the application metadata
type ApplicationConfig struct {
	Name        string   `yaml:"name" json:"name"`
	Version     string   `yaml:"version,omitempty" json:"version,omitempty"`
	Description string   `yaml:"description,omitempty" json:"description,omitempty"`
	Authors     []string `yaml:"authors,omitempty" json:"authors,omitempty"`
}

// ComponentConfig defines a component in the application
type ComponentConfig struct {
	ID                   string                 `yaml:"id" json:"id"`
	Source               interface{}            `yaml:"source" json:"source"` // Can be string or map for different source types
	Description          string                 `yaml:"description,omitempty" json:"description,omitempty"`
	Build                *BuildConfig           `yaml:"build,omitempty" json:"build,omitempty"`
	Variables            map[string]string      `yaml:"variables,omitempty" json:"variables,omitempty"`
	Files                []FileMount            `yaml:"files,omitempty" json:"files,omitempty"`
	AllowedOutboundHosts []string              `yaml:"allowed_outbound_hosts,omitempty" json:"allowed_outbound_hosts,omitempty"`
	AllowedHTTPHosts     []string              `yaml:"allowed_http_hosts,omitempty" json:"allowed_http_hosts,omitempty"`
	KeyValueStores       []string              `yaml:"key_value_stores,omitempty" json:"key_value_stores,omitempty"`
	SQLiteDatabases      []string              `yaml:"sqlite_databases,omitempty" json:"sqlite_databases,omitempty"`
	AIModels            []string              `yaml:"ai_models,omitempty" json:"ai_models,omitempty"`
}

// BuildConfig defines build configuration for a component
type BuildConfig struct {
	Command string   `yaml:"command,omitempty" json:"command,omitempty"`
	Workdir string   `yaml:"workdir,omitempty" json:"workdir,omitempty"`
	Watch   []string `yaml:"watch,omitempty" json:"watch,omitempty"`
}

// FileMount defines a file mount for a component
type FileMount struct {
	Source      string `yaml:"source" json:"source"`
	Destination string `yaml:"destination" json:"destination"`
}

// TriggerConfig defines a trigger for a component
type TriggerConfig struct {
	Type      string           `yaml:"type" json:"type"` // http, redis
	Component string           `yaml:"component" json:"component"`
	Route     string           `yaml:"route,omitempty" json:"route,omitempty"`         // For HTTP triggers
	Channel   string           `yaml:"channel,omitempty" json:"channel,omitempty"`     // For Redis triggers
	Executor  *ExecutorConfig  `yaml:"executor,omitempty" json:"executor,omitempty"`
}

// ExecutorConfig defines the executor for a trigger
type ExecutorConfig struct {
	Type string `yaml:"type,omitempty" json:"type,omitempty"` // spin, wagi
}

// MCPConfig defines MCP-specific configuration
type MCPConfig struct {
	Gateway    *GatewayConfig    `yaml:"gateway,omitempty" json:"gateway,omitempty"`
	Authorizer *AuthorizerConfig `yaml:"authorizer,omitempty" json:"authorizer,omitempty"`
}

// GatewayConfig defines MCP gateway configuration
type GatewayConfig struct {
	Enabled   bool   `yaml:"enabled" json:"enabled"`
	Component string `yaml:"component,omitempty" json:"component,omitempty"` // Override default gateway
}

// AuthorizerConfig defines MCP authorizer configuration
type AuthorizerConfig struct {
	Enabled       bool                   `yaml:"enabled" json:"enabled"`
	Component     string                 `yaml:"component,omitempty" json:"component,omitempty"` // Override default authorizer
	AccessControl string                 `yaml:"access_control,omitempty" json:"access_control,omitempty"` // public, private, org, custom
	JWTIssuer     string                 `yaml:"jwt_issuer,omitempty" json:"jwt_issuer,omitempty"`
	JWTAudience   string                 `yaml:"jwt_audience,omitempty" json:"jwt_audience,omitempty"`
	AllowedRoles  []string               `yaml:"allowed_roles,omitempty" json:"allowed_roles,omitempty"`
	OrgID         string                 `yaml:"org_id,omitempty" json:"org_id,omitempty"`
}

// DeploymentRequest represents a deployment request to the platform
// This combines the FTL config with registry references for pushed components
type DeploymentRequest struct {
	Config      FTLConfig            `json:"config"`
	Components  []ComponentReference `json:"components"`
	Environment string              `json:"environment,omitempty"`
}

// ComponentReference provides registry information for a pushed component
type ComponentReference struct {
	ID          string `json:"id"`           // Component ID from config
	RegistryURI string `json:"registry_uri"` // Full registry URI
	Digest      string `json:"digest"`       // Content digest
}

// DeploymentResponse represents the platform's response to a deployment request
type DeploymentResponse struct {
	DeploymentID string    `json:"deployment_id"`
	Status       string    `json:"status"`
	Message      string    `json:"message,omitempty"`
	AppURL       string    `json:"app_url,omitempty"`
	CreatedAt    time.Time `json:"created_at"`
}

// Default values
const (
	DefaultVersion = "0.1.0"
	
	// Access control modes
	AccessControlPublic  = "public"
	AccessControlPrivate = "private"
	AccessControlOrg     = "org"
	AccessControlCustom  = "custom"
	
	// Trigger types
	TriggerTypeHTTP  = "http"
	TriggerTypeRedis = "redis"
	
	// Executor types
	ExecutorTypeSpin = "spin"
	ExecutorTypeWagi = "wagi"
)

// Validate performs basic validation on the FTL config
func (c *FTLConfig) Validate() error {
	if c.Application.Name == "" {
		return fmt.Errorf("application name is required")
	}
	
	// Ensure version is set
	if c.Application.Version == "" {
		c.Application.Version = DefaultVersion
	}
	
	// Validate components
	componentIDs := make(map[string]bool)
	for _, comp := range c.Components {
		if comp.ID == "" {
			return fmt.Errorf("component ID is required")
		}
		if comp.Source == "" {
			return fmt.Errorf("component source is required for %s", comp.ID)
		}
		if componentIDs[comp.ID] {
			return fmt.Errorf("duplicate component ID: %s", comp.ID)
		}
		componentIDs[comp.ID] = true
	}
	
	// Validate triggers reference existing components
	for _, trigger := range c.Triggers {
		if trigger.Type == "" {
			return fmt.Errorf("trigger type is required")
		}
		if trigger.Component == "" {
			return fmt.Errorf("trigger component is required")
		}
		if !componentIDs[trigger.Component] {
			return fmt.Errorf("trigger references unknown component: %s", trigger.Component)
		}
		
		// Validate trigger-specific fields
		if trigger.Type == TriggerTypeHTTP && trigger.Route == "" {
			return fmt.Errorf("HTTP trigger requires route")
		}
		if trigger.Type == TriggerTypeRedis && trigger.Channel == "" {
			return fmt.Errorf("Redis trigger requires channel")
		}
	}
	
	// Validate MCP config if present
	if c.MCP != nil && c.MCP.Authorizer != nil {
		auth := c.MCP.Authorizer
		if auth.AccessControl == AccessControlCustom {
			if auth.JWTIssuer == "" {
				return fmt.Errorf("custom access control requires JWT issuer")
			}
		}
		if auth.AccessControl == AccessControlOrg && auth.OrgID == "" {
			// OrgID can be determined at deployment time if not specified
			// So this is not an error
		}
	}
	
	return nil
}

// SetDefaults sets default values for optional fields
func (c *FTLConfig) SetDefaults() {
	if c.Application.Version == "" {
		c.Application.Version = DefaultVersion
	}
	
	// Set default MCP config if not specified
	if c.MCP == nil {
		c.MCP = &MCPConfig{
			Gateway: &GatewayConfig{
				Enabled: true,
			},
			Authorizer: &AuthorizerConfig{
				Enabled:       true,
				AccessControl: AccessControlPublic,
			},
		}
	} else {
		if c.MCP.Gateway == nil {
			c.MCP.Gateway = &GatewayConfig{Enabled: true}
		}
		if c.MCP.Authorizer == nil {
			c.MCP.Authorizer = &AuthorizerConfig{
				Enabled:       true,
				AccessControl: AccessControlPublic,
			}
		}
	}
}

// IsRegistrySource returns true if the source is a registry reference
func (c *ComponentConfig) IsRegistrySource() bool {
	// Check if source is a map (registry or URL source)
	if _, isMap := c.Source.(map[string]interface{}); isMap {
		return true
	}
	
	// If it's a string, it's a local path
	return false
}

// IsLocalSource returns true if the source is a local file path
func (c *ComponentConfig) IsLocalSource() bool {
	_, isString := c.Source.(string)
	return isString
}