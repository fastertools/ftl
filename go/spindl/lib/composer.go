// Package lib provides the spindk library API for programmatic use
package lib

import (
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/fastertools/ftl-cli/go/shared/config"
	"github.com/fastertools/ftl-cli/go/spindl/internal/synth"
	"gopkg.in/yaml.v3"
)

// Composer provides the main API for synthesizing Spin configurations
type Composer struct {
	engine *synth.Engine
	config *config.FTLConfig
	overlays []*config.FTLConfig
	variables map[string]string
}

// New creates a new Composer instance
func New() *Composer {
	return &Composer{
		engine: synth.NewEngine(),
		variables: make(map[string]string),
	}
}

// Option is a functional option for configuring the Composer
type Option func(*Composer) error

// WithConfig loads the base configuration from a reader
func WithConfig(r io.Reader) Option {
	return func(c *Composer) error {
		data, err := io.ReadAll(r)
		if err != nil {
			return fmt.Errorf("failed to read config: %w", err)
		}

		var cfg config.FTLConfig
		if err := yaml.Unmarshal(data, &cfg); err != nil {
			return fmt.Errorf("failed to parse config: %w", err)
		}

		c.config = &cfg
		return nil
	}
}

// WithConfigStruct uses an existing FTLConfig struct
func WithConfigStruct(cfg *config.FTLConfig) Option {
	return func(c *Composer) error {
		c.config = cfg
		return nil
	}
}

// WithOverlay adds an overlay configuration from a reader
func WithOverlay(r io.Reader) Option {
	return func(c *Composer) error {
		data, err := io.ReadAll(r)
		if err != nil {
			return fmt.Errorf("failed to read overlay: %w", err)
		}

		var overlay config.FTLConfig
		if err := yaml.Unmarshal(data, &overlay); err != nil {
			return fmt.Errorf("failed to parse overlay: %w", err)
		}

		c.overlays = append(c.overlays, &overlay)
		return nil
	}
}

// WithOverlayStruct adds an overlay configuration from a struct
func WithOverlayStruct(overlay *config.FTLConfig) Option {
	return func(c *Composer) error {
		c.overlays = append(c.overlays, overlay)
		return nil
	}
}

// WithOverlayFile adds an overlay configuration from a file path
func WithOverlayFile(path string) Option {
	return func(c *Composer) error {
		f, err := os.Open(path)
		if err != nil {
			return fmt.Errorf("failed to open overlay file: %w", err)
		}
		defer f.Close()

		return WithOverlay(f)(c)
	}
}

// WithVariable sets a configuration variable
func WithVariable(key, value string) Option {
	return func(c *Composer) error {
		c.variables[key] = value
		return nil
	}
}

// WithVariables sets multiple configuration variables
func WithVariables(vars map[string]string) Option {
	return func(c *Composer) error {
		for k, v := range vars {
			c.variables[k] = v
		}
		return nil
	}
}

// Compose synthesizes the final configuration with all options applied
func (c *Composer) Compose(opts ...Option) (string, error) {
	// Apply options
	for _, opt := range opts {
		if err := opt(c); err != nil {
			return "", err
		}
	}

	// Validate we have a base config
	if c.config == nil {
		return "", fmt.Errorf("no base configuration provided")
	}

	// Merge configurations: base < overlays < variables
	merged := c.mergeConfigurations()

	// Validate the merged configuration
	if err := merged.Validate(); err != nil {
		return "", fmt.Errorf("invalid configuration: %w", err)
	}

	// Set defaults
	merged.SetDefaults()

	// Synthesize to spin.toml
	spinToml, err := c.engine.SynthesizeFromConfig(merged)
	if err != nil {
		return "", fmt.Errorf("failed to synthesize: %w", err)
	}

	return spinToml, nil
}

// ComposeToFile synthesizes and writes the result to a file
func (c *Composer) ComposeToFile(path string, opts ...Option) error {
	result, err := c.Compose(opts...)
	if err != nil {
		return err
	}

	return os.WriteFile(path, []byte(result), 0644)
}

// mergeConfigurations merges base config with overlays and variables
func (c *Composer) mergeConfigurations() *config.FTLConfig {
	// Start with base config (deep copy)
	merged := c.deepCopyConfig(c.config)

	// Apply each overlay in order
	for _, overlay := range c.overlays {
		c.mergeOverlay(merged, overlay)
	}

	// Apply variables
	c.applyVariables(merged)

	return merged
}

// deepCopyConfig creates a deep copy of the configuration
func (c *Composer) deepCopyConfig(cfg *config.FTLConfig) *config.FTLConfig {
	// For now, use YAML marshal/unmarshal for deep copy
	// In production, use a proper deep copy library
	data, _ := yaml.Marshal(cfg)
	var copy config.FTLConfig
	yaml.Unmarshal(data, &copy)
	return &copy
}

// mergeOverlay merges an overlay configuration into the base
func (c *Composer) mergeOverlay(base, overlay *config.FTLConfig) {
	// Merge application metadata
	if overlay.Application.Name != "" {
		base.Application.Name = overlay.Application.Name
	}
	if overlay.Application.Version != "" {
		base.Application.Version = overlay.Application.Version
	}

	// Merge components (additive)
	base.Components = append(base.Components, overlay.Components...)

	// Merge triggers (overlay replaces if present)
	if len(overlay.Triggers) > 0 {
		base.Triggers = overlay.Triggers
	}

	// Merge variables
	if base.Variables == nil {
		base.Variables = make(map[string]string)
	}
	for k, v := range overlay.Variables {
		base.Variables[k] = v
	}

	// Merge MCP config
	if overlay.MCP != nil {
		if base.MCP == nil {
			base.MCP = overlay.MCP
		} else {
			// Merge MCP fields
			if overlay.MCP.Gateway != nil {
				base.MCP.Gateway = overlay.MCP.Gateway
			}
			if overlay.MCP.Authorizer != nil {
				// Merge authorizer config
				if base.MCP.Authorizer == nil {
					base.MCP.Authorizer = overlay.MCP.Authorizer
				} else {
					// Merge individual authorizer fields
					if overlay.MCP.Authorizer.AccessControl != "" {
						base.MCP.Authorizer.AccessControl = overlay.MCP.Authorizer.AccessControl
					}
					if overlay.MCP.Authorizer.JWTIssuer != "" {
						base.MCP.Authorizer.JWTIssuer = overlay.MCP.Authorizer.JWTIssuer
					}
					// ... merge other fields as needed
				}
			}
		}
	}
}

// applyVariables applies variable overrides to the configuration
func (c *Composer) applyVariables(cfg *config.FTLConfig) {
	if cfg.Variables == nil {
		cfg.Variables = make(map[string]string)
	}

	// Override with composer variables
	for k, v := range c.variables {
		cfg.Variables[k] = v
	}

	// Apply variable substitution in component environment vars
	for i := range cfg.Components {
		if cfg.Components[i].Environment == nil {
			continue
		}
		for envKey, envVal := range cfg.Components[i].Environment {
			cfg.Components[i].Environment[envKey] = c.substituteVariables(envVal)
		}
	}
}

// substituteVariables replaces {{ variable }} placeholders
func (c *Composer) substituteVariables(value string) string {
	// Simple implementation - in production use a template engine
	result := value
	for k, v := range c.variables {
		placeholder := fmt.Sprintf("{{ %s }}", k)
		result = strings.ReplaceAll(result, placeholder, v)
	}
	return result
}