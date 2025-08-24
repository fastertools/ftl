package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// TestConfig provides standardized test configuration
type TestConfig struct {
	mu sync.RWMutex

	// Test Environment
	TestMode    bool   `json:"test_mode"`
	TestRunID   string `json:"test_run_id"`
	Environment string `json:"environment"` // "local", "ci", "staging"

	// Server Configuration
	ServerPort     int    `json:"server_port"`
	ServerHost     string `json:"server_host"`
	ServerProtocol string `json:"server_protocol"`
	BaseURL        string `json:"base_url"`

	// File Paths
	ProjectsFile    string `json:"projects_file"`
	TestDataDir     string `json:"test_data_dir"`
	TempDir         string `json:"temp_dir"`
	LogDir          string `json:"log_dir"`
	ScreenshotDir   string `json:"screenshot_dir"`

	// Test Data Configuration
	DefaultProjectType     string `json:"default_project_type"`
	DefaultProjectLanguage string `json:"default_project_language"`
	DefaultTimeout         int    `json:"default_timeout_ms"`
	PollInterval           int    `json:"poll_interval_ms"`

	// Feature Flags
	EnableWatchMode      bool `json:"enable_watch_mode"`
	EnableDebugLogging   bool `json:"enable_debug_logging"`
	EnableScreenshots    bool `json:"enable_screenshots"`
	EnablePerformanceLog bool `json:"enable_performance_log"`

	// Test Metadata
	TestSuite   string    `json:"test_suite"`
	TestFile    string    `json:"test_file"`
	TestName    string    `json:"test_name"`
	StartedAt   time.Time `json:"started_at"`
	LastUpdated time.Time `json:"last_updated"`
}

var (
	testInstance *TestConfig
	testOnce     sync.Once
)

// GetTestConfig returns the singleton test configuration
func GetTestConfig() *TestConfig {
	testOnce.Do(func() {
		testInstance = &TestConfig{
			// Default values
			TestMode:       os.Getenv("FTL_TEST_MODE") == "true",
			TestRunID:      generateTestRunID(),
			Environment:    detectEnvironment(),
			ServerPort:     8080,
			ServerHost:     "localhost",
			ServerProtocol: "http",
			BaseURL:        "http://localhost:8080",
			
			// Default paths
			ProjectsFile:  ".e2e-projects.json",
			TestDataDir:   ".e2e-projects",
			TempDir:       filepath.Join(os.TempDir(), "ftl-tests"),
			LogDir:        filepath.Join(".e2e-projects", "logs"),
			ScreenshotDir: filepath.Join("e2e-tests", "screenshots"),
			
			// Default test data
			DefaultProjectType:     "tool",
			DefaultProjectLanguage: "rust",
			DefaultTimeout:         30000,
			PollInterval:           1000,
			
			// Default feature flags
			EnableWatchMode:      true,
			EnableDebugLogging:   os.Getenv("DEBUG") == "true",
			EnableScreenshots:    true,
			EnablePerformanceLog: false,
			
			// Metadata
			StartedAt:   time.Now(),
			LastUpdated: time.Now(),
		}
		
		// Override from environment
		testInstance.loadFromEnvironment()
		
		// Override from config file if exists
		testInstance.loadFromFile()
	})
	return testInstance
}

// Update updates the test configuration
func (tc *TestConfig) Update(updates map[string]interface{}) error {
	tc.mu.Lock()
	defer tc.mu.Unlock()
	
	data, err := json.Marshal(tc)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}
	
	var configMap map[string]interface{}
	if err := json.Unmarshal(data, &configMap); err != nil {
		return fmt.Errorf("failed to unmarshal config: %w", err)
	}
	
	// Apply updates
	for key, value := range updates {
		configMap[key] = value
	}
	
	// Marshal back to struct
	updatedData, err := json.Marshal(configMap)
	if err != nil {
		return fmt.Errorf("failed to marshal updates: %w", err)
	}
	
	if err := json.Unmarshal(updatedData, tc); err != nil {
		return fmt.Errorf("failed to apply updates: %w", err)
	}
	
	tc.LastUpdated = time.Now()
	return nil
}

// ToJSON returns the configuration as JSON
func (tc *TestConfig) ToJSON() ([]byte, error) {
	tc.mu.RLock()
	defer tc.mu.RUnlock()
	
	return json.MarshalIndent(tc, "", "  ")
}

// Reset resets the configuration to defaults
func (tc *TestConfig) Reset() {
	tc.mu.Lock()
	defer tc.mu.Unlock()
	
	// Reset to defaults
	*tc = TestConfig{
		TestMode:       os.Getenv("FTL_TEST_MODE") == "true",
		TestRunID:      generateTestRunID(),
		Environment:    detectEnvironment(),
		ServerPort:     8080,
		ServerHost:     "localhost",
		ServerProtocol: "http",
		BaseURL:        "http://localhost:8080",
		ProjectsFile:   ".e2e-projects.json",
		TestDataDir:    ".e2e-projects",
		TempDir:        filepath.Join(os.TempDir(), "ftl-tests"),
		LogDir:         filepath.Join(".e2e-projects", "logs"),
		ScreenshotDir:  filepath.Join("e2e-tests", "screenshots"),
		DefaultProjectType:     "tool",
		DefaultProjectLanguage: "rust",
		DefaultTimeout:         30000,
		PollInterval:           1000,
		EnableWatchMode:        true,
		EnableDebugLogging:     os.Getenv("DEBUG") == "true",
		EnableScreenshots:      true,
		EnablePerformanceLog:   false,
		StartedAt:              time.Now(),
		LastUpdated:            time.Now(),
	}
}

// loadFromEnvironment loads configuration from environment variables
func (tc *TestConfig) loadFromEnvironment() {
	// Server configuration
	if port := os.Getenv("FTL_TEST_PORT"); port != "" {
		fmt.Sscanf(port, "%d", &tc.ServerPort)
		tc.BaseURL = fmt.Sprintf("%s://%s:%d", tc.ServerProtocol, tc.ServerHost, tc.ServerPort)
	}

	if host := os.Getenv("FTL_TEST_HOST"); host != "" {
		tc.ServerHost = host
		tc.BaseURL = fmt.Sprintf("%s://%s:%d", tc.ServerProtocol, tc.ServerHost, tc.ServerPort)
	}

	// File paths
	if projectsFile := os.Getenv("FTL_PROJECTS_FILE"); projectsFile != "" {
		tc.ProjectsFile = projectsFile
	}

	if testDataDir := os.Getenv("FTL_TEST_DATA_DIR"); testDataDir != "" {
		tc.TestDataDir = testDataDir
	}

	// Test configuration
	if timeout := os.Getenv("FTL_TEST_TIMEOUT"); timeout != "" {
		fmt.Sscanf(timeout, "%d", &tc.DefaultTimeout)
	}

	if pollInterval := os.Getenv("FTL_POLL_INTERVAL"); pollInterval != "" {
		fmt.Sscanf(pollInterval, "%d", &tc.PollInterval)
	}

	// Feature flags
	if os.Getenv("FTL_ENABLE_WATCH") == "false" {
		tc.EnableWatchMode = false
	}

	if os.Getenv("FTL_ENABLE_SCREENSHOTS") == "false" {
		tc.EnableScreenshots = false
	}

	if os.Getenv("FTL_ENABLE_PERF_LOG") == "true" {
		tc.EnablePerformanceLog = true
	}

	// Test metadata
	if suite := os.Getenv("TEST_SUITE"); suite != "" {
		tc.TestSuite = suite
	}

	if file := os.Getenv("TEST_FILE"); file != "" {
		tc.TestFile = file
	}

	if name := os.Getenv("TEST_NAME"); name != "" {
		tc.TestName = name
	}
}

// loadFromFile loads configuration from a JSON file
func (tc *TestConfig) loadFromFile() {
	configPath := os.Getenv("FTL_TEST_CONFIG_FILE")
	if configPath == "" {
		configPath = "test-config.json"
	}
	
	data, err := os.ReadFile(configPath)
	if err != nil {
		// Config file is optional
		return
	}
	
	var fileConfig map[string]interface{}
	if err := json.Unmarshal(data, &fileConfig); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to parse test config file: %v\n", err)
		return
	}
	
	// Apply file configuration
	tc.Update(fileConfig)
}

// generateTestRunID generates a unique test run ID
func generateTestRunID() string {
	return fmt.Sprintf("test-run-%d", time.Now().Unix())
}

// detectEnvironment detects the current environment
func detectEnvironment() string {
	if os.Getenv("CI") == "true" {
		return "ci"
	}
	if os.Getenv("STAGING") == "true" {
		return "staging"
	}
	return "local"
}

// CreateTestProject creates a standardized test project configuration
func (tc *TestConfig) CreateTestProject(name string, overrides map[string]interface{}) map[string]interface{} {
	tc.mu.RLock()
	defer tc.mu.RUnlock()
	
	project := map[string]interface{}{
		"id":         fmt.Sprintf("project-%d-%s", time.Now().Unix(), name),
		"name":       name,
		"path":       filepath.Join(tc.TestDataDir, name),
		"type":       tc.DefaultProjectType,
		"language":   tc.DefaultProjectLanguage,
		"created_at": time.Now().Format(time.RFC3339),
		"updated_at": time.Now().Format(time.RFC3339),
		"status":     "inactive",
		"metadata": map[string]interface{}{
			"test_run_id": tc.TestRunID,
			"environment": tc.Environment,
			"created_by":  "test",
		},
	}
	
	// Apply overrides
	for key, value := range overrides {
		project[key] = value
	}
	
	return project
}

// GetProjectsFilePath returns the full path to the projects file
func (tc *TestConfig) GetProjectsFilePath() string {
	tc.mu.RLock()
	defer tc.mu.RUnlock()
	
	if filepath.IsAbs(tc.ProjectsFile) {
		return tc.ProjectsFile
	}
	return filepath.Join(".", tc.ProjectsFile)
}

// GetTestDataPath returns the full path to the test data directory
func (tc *TestConfig) GetTestDataPath() string {
	tc.mu.RLock()
	defer tc.mu.RUnlock()
	
	if filepath.IsAbs(tc.TestDataDir) {
		return tc.TestDataDir
	}
	return filepath.Join(".", tc.TestDataDir)
}

// EnsureDirectories ensures all required directories exist
func (tc *TestConfig) EnsureDirectories() error {
	tc.mu.RLock()
	defer tc.mu.RUnlock()
	
	dirs := []string{
		tc.GetTestDataPath(),
		tc.TempDir,
		tc.LogDir,
		tc.ScreenshotDir,
	}
	
	for _, dir := range dirs {
		if err := os.MkdirAll(dir, 0755); err != nil {
			return fmt.Errorf("failed to create directory %s: %w", dir, err)
		}
	}
	
	return nil
}

// Cleanup removes all test data
func (tc *TestConfig) Cleanup() error {
	tc.mu.Lock()
	defer tc.mu.Unlock()
	
	// Remove test data directory
	if err := os.RemoveAll(tc.GetTestDataPath()); err != nil {
		return fmt.Errorf("failed to remove test data: %w", err)
	}
	
	// Remove projects file
	if err := os.Remove(tc.GetProjectsFilePath()); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove projects file: %w", err)
	}
	
	// Remove temp directory
	if err := os.RemoveAll(tc.TempDir); err != nil {
		return fmt.Errorf("failed to remove temp directory: %w", err)
	}
	
	return nil
}