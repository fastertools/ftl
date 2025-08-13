package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFTLConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  *FTLConfig
		wantErr bool
		errMsg  string
	}{
		{
			name: "valid minimal config",
			config: &FTLConfig{
				Name: "my-app",
			},
			wantErr: false,
		},
		{
			name: "valid full config",
			config: &FTLConfig{
				Name:        "my-app",
				Version:     "1.0.0",
				Description: "Test app",
				Authors:     []string{"Test Author"},
				Variables:   map[string]string{"key": "value"},
			},
			wantErr: false,
		},
		{
			name:    "missing name",
			config:  &FTLConfig{},
			wantErr: true,
			errMsg:  "name is required",
		},
		{
			name: "invalid name - uppercase",
			config: &FTLConfig{
				Name: "MyApp",
			},
			wantErr: true,
			errMsg:  "invalid name",
		},
		{
			name: "invalid name - special chars",
			config: &FTLConfig{
				Name: "my_app",
			},
			wantErr: true,
			errMsg:  "invalid name",
		},
		{
			name: "invalid name - starts with hyphen",
			config: &FTLConfig{
				Name: "-myapp",
			},
			wantErr: true,
			errMsg:  "invalid name",
		},
		{
			name: "invalid name - ends with hyphen",
			config: &FTLConfig{
				Name: "myapp-",
			},
			wantErr: true,
			errMsg:  "invalid name",
		},
		{
			name: "invalid name - too long",
			config: &FTLConfig{
				Name: "this-is-a-very-long-name-that-exceeds-the-maximum-allowed-length-of-63",
			},
			wantErr: true,
			errMsg:  "invalid name",
		},
		{
			name: "valid with deploy config",
			config: &FTLConfig{
				Name: "my-app",
				Deploy: &DeployConfig{
					Environment: "production",
					Region:      "us-west-2",
				},
			},
			wantErr: false,
		},
		{
			name: "invalid deploy config",
			config: &FTLConfig{
				Name: "my-app",
				Deploy: &DeployConfig{
					Environment: "invalid",
				},
			},
			wantErr: true,
			errMsg:  "invalid environment",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestDeployConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  *DeployConfig
		wantErr bool
		errMsg  string
	}{
		{
			name:    "valid minimal - defaults to development",
			config:  &DeployConfig{},
			wantErr: false,
		},
		{
			name: "valid production",
			config: &DeployConfig{
				Environment: "production",
			},
			wantErr: false,
		},
		{
			name: "valid staging",
			config: &DeployConfig{
				Environment: "staging",
			},
			wantErr: false,
		},
		{
			name: "invalid environment",
			config: &DeployConfig{
				Environment: "testing",
			},
			wantErr: true,
			errMsg:  "invalid environment",
		},
		{
			name: "valid with auth",
			config: &DeployConfig{
				Environment: "production",
				Auth: &AuthConfig{
					Type:  "token",
					Token: "secret-token",
				},
			},
			wantErr: false,
		},
		{
			name: "invalid auth",
			config: &DeployConfig{
				Environment: "production",
				Auth: &AuthConfig{
					Type: "invalid",
				},
			},
			wantErr: true,
			errMsg:  "invalid auth type",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
				// Check default is applied
				if tt.config.Environment == "" {
					assert.Equal(t, "development", tt.config.Environment)
				}
			}
		})
	}
}

func TestAuthConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  *AuthConfig
		wantErr bool
		errMsg  string
	}{
		{
			name: "valid token auth",
			config: &AuthConfig{
				Type:  "token",
				Token: "my-secret-token",
			},
			wantErr: false,
		},
		{
			name: "valid oauth",
			config: &AuthConfig{
				Type:     "oauth",
				ClientID: "client-123",
				Issuer:   "https://auth.example.com",
			},
			wantErr: false,
		},
		{
			name: "valid none",
			config: &AuthConfig{
				Type: "none",
			},
			wantErr: false,
		},
		{
			name: "invalid type",
			config: &AuthConfig{
				Type: "basic",
			},
			wantErr: true,
			errMsg:  "invalid auth type",
		},
		{
			name: "token auth missing token",
			config: &AuthConfig{
				Type: "token",
			},
			wantErr: true,
			errMsg:  "token is required",
		},
		{
			name: "oauth missing client_id",
			config: &AuthConfig{
				Type:   "oauth",
				Issuer: "https://auth.example.com",
			},
			wantErr: true,
			errMsg:  "client_id is required",
		},
		{
			name: "oauth missing issuer",
			config: &AuthConfig{
				Type:     "oauth",
				ClientID: "client-123",
			},
			wantErr: true,
			errMsg:  "issuer is required",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestLoad(t *testing.T) {
	// Create temp directory for test files
	tmpDir := t.TempDir()

	tests := []struct {
		name        string
		filename    string
		content     string
		wantErr     bool
		errMsg      string
		validateCfg func(*testing.T, *FTLConfig)
	}{
		{
			name:     "valid YAML",
			filename: "ftl.yaml",
			content: `name: test-app
version: 1.0.0
description: Test application
authors:
  - John Doe
variables:
  env: test`,
			wantErr: false,
			validateCfg: func(t *testing.T, cfg *FTLConfig) {
				assert.Equal(t, "test-app", cfg.Name)
				assert.Equal(t, "1.0.0", cfg.Version)
				assert.Equal(t, "Test application", cfg.Description)
				assert.Equal(t, []string{"John Doe"}, cfg.Authors)
				assert.Equal(t, "test", cfg.Variables["env"])
			},
		},
		{
			name:     "valid TOML",
			filename: "ftl.toml",
			content: `name = "test-app"
version = "1.0.0"
description = "Test application"
authors = ["John Doe"]

[variables]
env = "test"`,
			wantErr: false,
			validateCfg: func(t *testing.T, cfg *FTLConfig) {
				assert.Equal(t, "test-app", cfg.Name)
				assert.Equal(t, "1.0.0", cfg.Version)
				assert.Equal(t, "Test application", cfg.Description)
				assert.Equal(t, []string{"John Doe"}, cfg.Authors)
				assert.Equal(t, "test", cfg.Variables["env"])
			},
		},
		{
			name:     "invalid YAML syntax",
			filename: "ftl.yaml",
			content:  "name: test-app\n- invalid\n  : structure:\n",
			wantErr:  true,
			errMsg:   "failed to parse YAML",
		},
		{
			name:     "invalid TOML syntax",
			filename: "ftl.toml",
			content:  `name = "test-app"\ninvalid toml content`,
			wantErr:  true,
			errMsg:   "failed to parse TOML",
		},
		{
			name:     "missing required field",
			filename: "ftl.yaml",
			content:  `version: 1.0.0`,
			wantErr:  true,
			errMsg:   "name is required",
		},
		{
			name:     "unsupported format",
			filename: "ftl.json",
			content:  `{"name": "test-app"}`,
			wantErr:  true,
			errMsg:   "unsupported config format",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create test file
			testFile := filepath.Join(tmpDir, tt.filename)
			err := os.WriteFile(testFile, []byte(tt.content), 0644)
			require.NoError(t, err)

			// Load config
			cfg, err := Load(testFile)
			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
			} else {
				assert.NoError(t, err)
				require.NotNil(t, cfg)
				if tt.validateCfg != nil {
					tt.validateCfg(t, cfg)
				}
			}
		})
	}
}

func TestSave(t *testing.T) {
	tmpDir := t.TempDir()

	cfg := &FTLConfig{
		Name:        "test-app",
		Version:     "1.0.0",
		Description: "Test application",
		Authors:     []string{"Jane Doe"},
		Variables: map[string]string{
			"key": "value",
		},
	}

	tests := []struct {
		name     string
		filename string
		wantErr  bool
	}{
		{
			name:     "save as YAML",
			filename: "ftl.yaml",
			wantErr:  false,
		},
		{
			name:     "save as TOML",
			filename: "ftl.toml",
			wantErr:  false,
		},
		{
			name:     "unsupported format",
			filename: "ftl.json",
			wantErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			testFile := filepath.Join(tmpDir, tt.filename)
			err := cfg.Save(testFile)
			
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				
				// Verify file was created and can be loaded
				loadedCfg, err := Load(testFile)
				require.NoError(t, err)
				assert.Equal(t, cfg.Name, loadedCfg.Name)
				assert.Equal(t, cfg.Version, loadedCfg.Version)
				assert.Equal(t, cfg.Description, loadedCfg.Description)
				assert.Equal(t, cfg.Authors, loadedCfg.Authors)
				assert.Equal(t, cfg.Variables, loadedCfg.Variables)
			}
		})
	}
}

func TestIsValidName(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  bool
	}{
		{"valid simple", "myapp", true},
		{"valid with hyphen", "my-app", true},
		{"valid with numbers", "app123", true},
		{"valid complex", "my-app-123", true},
		{"invalid uppercase", "MyApp", false},
		{"invalid underscore", "my_app", false},
		{"invalid special", "my@app", false},
		{"invalid space", "my app", false},
		{"invalid starts hyphen", "-myapp", false},
		{"invalid ends hyphen", "myapp-", false},
		{"invalid empty", "", false},
		{"invalid too long", "a123456789012345678901234567890123456789012345678901234567890123", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := isValidName(tt.input)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestContains(t *testing.T) {
	tests := []struct {
		name  string
		slice []string
		value string
		want  bool
	}{
		{"found", []string{"a", "b", "c"}, "b", true},
		{"not found", []string{"a", "b", "c"}, "d", false},
		{"empty slice", []string{}, "a", false},
		{"empty value", []string{"a", "b"}, "", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := contains(tt.slice, tt.value)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestFindConfigFile(t *testing.T) {
	// Save current dir and restore after test
	origDir, _ := os.Getwd()
	defer os.Chdir(origDir)

	tmpDir := t.TempDir()
	os.Chdir(tmpDir)

	// Test default when no file exists
	assert.Equal(t, "ftl.yaml", findConfigFile())

	// Test finding ftl.yaml
	os.WriteFile("ftl.yaml", []byte("test"), 0644)
	assert.Equal(t, "ftl.yaml", findConfigFile())

	// Test preferring ftl.yaml over ftl.toml
	os.WriteFile("ftl.toml", []byte("test"), 0644)
	assert.Equal(t, "ftl.yaml", findConfigFile())

	// Test finding ftl.toml when ftl.yaml doesn't exist
	os.Remove("ftl.yaml")
	assert.Equal(t, "ftl.toml", findConfigFile())
}