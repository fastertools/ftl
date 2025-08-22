// Package config manages user-level configuration for the FTL CLI
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
)

// Config represents the user's FTL CLI configuration
type Config struct {
	// CurrentOrg is the currently selected organization
	CurrentOrg string `json:"current_org,omitempty"`

	// DefaultEnvironment is the default deployment environment
	DefaultEnvironment string `json:"default_environment,omitempty"`

	// Organizations stores metadata about known organizations
	Organizations map[string]OrgInfo `json:"organizations,omitempty"`

	// Preferences stores user preferences
	Preferences Preferences `json:"preferences,omitempty"`

	// LastUpdateCheck tracks when we last checked for updates
	LastUpdateCheck string `json:"last_update_check,omitempty"`

	// CurrentUser stores info about the logged-in user
	CurrentUser *UserInfo `json:"current_user,omitempty"`

	// Version of the config schema
	Version string `json:"version"`
}

// UserInfo stores information about the authenticated user
type UserInfo struct {
	Username  string `json:"username,omitempty"`
	Email     string `json:"email,omitempty"`
	UserID    string `json:"user_id,omitempty"`
	UpdatedAt string `json:"updated_at,omitempty"`
}

// OrgInfo stores information about an organization
type OrgInfo struct {
	ID          string `json:"id"`
	Name        string `json:"name,omitempty"`
	LastUsed    string `json:"last_used,omitempty"`
	IsDefault   bool   `json:"is_default,omitempty"`
	Environment string `json:"environment,omitempty"` // Default env for this org
}

// Preferences stores user preferences
type Preferences struct {
	// ColorOutput controls whether to use colored output
	ColorOutput bool `json:"color_output"`

	// Verbose controls verbose output
	Verbose bool `json:"verbose"`

	// AutoUpdate controls automatic update checks
	AutoUpdate bool `json:"auto_update"`

	// ConfirmDeploy controls whether to confirm before deploying
	ConfirmDeploy bool `json:"confirm_deploy"`
}

var (
	instance *Config
	once     sync.Once
	mu       sync.RWMutex
)

// configPath returns the path to the config file
func configPath() (string, error) {
	var configDir string

	// Check XDG_CONFIG_HOME first for testing and Linux compatibility
	if xdgConfig := os.Getenv("XDG_CONFIG_HOME"); xdgConfig != "" {
		configDir = xdgConfig
	} else {
		// Fall back to os.UserConfigDir() for platform-specific defaults
		var err error
		configDir, err = os.UserConfigDir()
		if err != nil {
			return "", fmt.Errorf("failed to get config directory: %w", err)
		}
	}

	ftlDir := filepath.Join(configDir, "ftl")
	return filepath.Join(ftlDir, "config.json"), nil
}

// UserDataPath returns the path for user data files (projects.json, etc.)
func UserDataPath(filename string) (string, error) {
	var dataDir string

	// Check XDG_DATA_HOME first for testing and Linux compatibility
	if xdgData := os.Getenv("XDG_DATA_HOME"); xdgData != "" {
		dataDir = xdgData
	} else {
		// Fall back to os.UserConfigDir() for platform-specific defaults
		// This works for data storage on most platforms:
		// macOS: ~/Library/Application Support
		// Windows: %APPDATA%
		// Linux: ~/.config (but XDG_DATA_HOME should be set to ~/.local/share)
		var err error
		dataDir, err = os.UserConfigDir()
		if err != nil {
			return "", fmt.Errorf("failed to get data directory: %w", err)
		}
	}

	ftlDir := filepath.Join(dataDir, "ftl")
	return filepath.Join(ftlDir, filename), nil
}

// Load loads the configuration from disk or creates a new one
func Load() (*Config, error) {
	var err error
	once.Do(func() {
		instance, err = load()
	})

	if err != nil {
		return nil, err
	}

	return instance, nil
}

// load reads the config from disk or creates default
func load() (*Config, error) {
	path, err := configPath()
	if err != nil {
		return nil, err
	}

	// Ensure directory exists
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0750); err != nil {
		return nil, fmt.Errorf("failed to create config directory: %w", err)
	}

	// Try to read existing config
	data, err := os.ReadFile(path) // #nosec G304 - path is controlled via configPath()
	if err != nil {
		if os.IsNotExist(err) {
			// Create default config
			return defaultConfig(), nil
		}
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	// Ensure maps are initialized
	if cfg.Organizations == nil {
		cfg.Organizations = make(map[string]OrgInfo)
	}

	return &cfg, nil
}

// defaultConfig returns a default configuration
func defaultConfig() *Config {
	return &Config{
		Version:       "1.0",
		Organizations: make(map[string]OrgInfo),
		Preferences: Preferences{
			ColorOutput:   true,
			Verbose:       false,
			AutoUpdate:    true,
			ConfirmDeploy: true,
		},
		DefaultEnvironment: "production",
	}
}

// Save saves the configuration to disk
func (c *Config) Save() error {
	mu.Lock()
	defer mu.Unlock()

	path, err := configPath()
	if err != nil {
		return err
	}

	// Ensure directory exists
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0750); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	// Marshal with indentation for readability
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	// Write atomically by writing to temp file then renaming
	tempPath := path + ".tmp"
	if err := os.WriteFile(tempPath, data, 0600); err != nil {
		return fmt.Errorf("failed to write config: %w", err)
	}

	if err := os.Rename(tempPath, path); err != nil {
		// Clean up temp file on error
		_ = os.Remove(tempPath)
		return fmt.Errorf("failed to save config: %w", err)
	}

	return nil
}

// GetCurrentOrg returns the currently selected organization
func (c *Config) GetCurrentOrg() string {
	mu.RLock()
	defer mu.RUnlock()
	return c.CurrentOrg
}

// SetCurrentOrg sets the currently selected organization
func (c *Config) SetCurrentOrg(orgID string) error {
	mu.Lock()
	c.CurrentOrg = orgID
	mu.Unlock()

	return c.Save()
}

// AddOrganization adds or updates organization info
func (c *Config) AddOrganization(info OrgInfo) error {
	mu.Lock()
	if c.Organizations == nil {
		c.Organizations = make(map[string]OrgInfo)
	}
	c.Organizations[info.ID] = info
	mu.Unlock()

	return c.Save()
}

// GetOrganization retrieves organization info
func (c *Config) GetOrganization(orgID string) (OrgInfo, bool) {
	mu.RLock()
	defer mu.RUnlock()

	info, exists := c.Organizations[orgID]
	return info, exists
}

// ListOrganizations returns all known organizations
func (c *Config) ListOrganizations() []OrgInfo {
	mu.RLock()
	defer mu.RUnlock()

	orgs := make([]OrgInfo, 0, len(c.Organizations))
	for _, org := range c.Organizations {
		orgs = append(orgs, org)
	}

	return orgs
}

// GetDefaultEnvironment returns the default environment for deployments
func (c *Config) GetDefaultEnvironment() string {
	mu.RLock()
	defer mu.RUnlock()

	if c.DefaultEnvironment == "" {
		return "production"
	}
	return c.DefaultEnvironment
}

// SetDefaultEnvironment sets the default environment
func (c *Config) SetDefaultEnvironment(env string) error {
	mu.Lock()
	c.DefaultEnvironment = env
	mu.Unlock()

	return c.Save()
}

// Reset resets the configuration to defaults
func (c *Config) Reset() error {
	mu.Lock()
	defer mu.Unlock()

	*c = *defaultConfig()
	return c.Save()
}

// GetCurrentUser returns the current user info
func (c *Config) GetCurrentUser() *UserInfo {
	mu.RLock()
	defer mu.RUnlock()
	return c.CurrentUser
}

// SetCurrentUser updates the current user info
func (c *Config) SetCurrentUser(user *UserInfo) error {
	mu.Lock()
	c.CurrentUser = user
	mu.Unlock()

	return c.Save()
}

// ClearCurrentUser removes the current user info
func (c *Config) ClearCurrentUser() error {
	mu.Lock()
	c.CurrentUser = nil
	mu.Unlock()

	return c.Save()
}
