package config

import (
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"testing"
)

func TestConfigLoadSave(t *testing.T) {
	// Use temp directory for testing
	tmpDir := t.TempDir()
	oldHome := os.Getenv("HOME")
	oldUserConfig := os.Getenv("XDG_CONFIG_HOME")

	// Set test environment
	_ = os.Setenv("XDG_CONFIG_HOME", tmpDir)
	defer func() {
		_ = os.Setenv("HOME", oldHome)
		if oldUserConfig != "" {
			_ = os.Setenv("XDG_CONFIG_HOME", oldUserConfig)
		} else {
			_ = os.Unsetenv("XDG_CONFIG_HOME")
		}
	}()

	// Reset singleton for testing
	instance = nil
	once = sync.Once{}

	// Test loading default config
	cfg, err := Load()
	if err != nil {
		t.Fatalf("Failed to load default config: %v", err)
	}

	if cfg.Version != "1.0" {
		t.Errorf("Expected version 1.0, got %s", cfg.Version)
	}

	// Test setting current org
	testOrgID := "org_test123"
	err = cfg.SetCurrentOrg(testOrgID)
	if err != nil {
		t.Fatalf("Failed to set current org: %v", err)
	}

	if cfg.GetCurrentOrg() != testOrgID {
		t.Errorf("Expected current org %s, got %s", testOrgID, cfg.GetCurrentOrg())
	}

	// Verify file was created
	configPath := filepath.Join(tmpDir, "ftl", "config.json")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Error("Config file was not created")
	}

	// Reset and reload to verify persistence
	instance = nil
	once = sync.Once{}

	cfg2, err := Load()
	if err != nil {
		t.Fatalf("Failed to reload config: %v", err)
	}

	if cfg2.GetCurrentOrg() != testOrgID {
		t.Errorf("Current org not persisted, expected %s, got %s", testOrgID, cfg2.GetCurrentOrg())
	}
}

func TestOrganizationManagement(t *testing.T) {
	// Use temp directory
	tmpDir := t.TempDir()
	_ = os.Setenv("XDG_CONFIG_HOME", tmpDir)
	defer func() { _ = os.Unsetenv("XDG_CONFIG_HOME") }()

	// Reset singleton
	instance = nil
	once = sync.Once{}

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Failed to load config: %v", err)
	}

	// Test adding organization
	orgInfo := OrgInfo{
		ID:       "org_123",
		Name:     "Test Org",
		LastUsed: "2024-01-01T00:00:00Z",
	}

	err = cfg.AddOrganization(orgInfo)
	if err != nil {
		t.Fatalf("Failed to add organization: %v", err)
	}

	// Test retrieving organization
	retrieved, exists := cfg.GetOrganization("org_123")
	if !exists {
		t.Error("Organization not found after adding")
	}

	if retrieved.Name != "Test Org" {
		t.Errorf("Expected org name 'Test Org', got '%s'", retrieved.Name)
	}

	// Test listing organizations
	orgs := cfg.ListOrganizations()
	if len(orgs) != 1 {
		t.Errorf("Expected 1 organization, got %d", len(orgs))
	}
}

func TestDefaultEnvironment(t *testing.T) {
	// Use temp directory
	tmpDir := t.TempDir()
	_ = os.Setenv("XDG_CONFIG_HOME", tmpDir)
	defer func() { _ = os.Unsetenv("XDG_CONFIG_HOME") }()

	// Reset singleton
	instance = nil
	once = sync.Once{}

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Failed to load config: %v", err)
	}

	// Test default environment
	if cfg.GetDefaultEnvironment() != "production" {
		t.Errorf("Expected default environment 'production', got '%s'", cfg.GetDefaultEnvironment())
	}

	// Test setting environment
	err = cfg.SetDefaultEnvironment("staging")
	if err != nil {
		t.Fatalf("Failed to set environment: %v", err)
	}

	if cfg.GetDefaultEnvironment() != "staging" {
		t.Errorf("Expected environment 'staging', got '%s'", cfg.GetDefaultEnvironment())
	}
}

func TestConcurrency(t *testing.T) {
	// Use temp directory
	tmpDir := t.TempDir()
	_ = os.Setenv("XDG_CONFIG_HOME", tmpDir)
	defer func() { _ = os.Unsetenv("XDG_CONFIG_HOME") }()

	// Reset singleton
	instance = nil
	once = sync.Once{}

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Failed to load config: %v", err)
	}

	// Test concurrent access
	done := make(chan bool, 10)

	// Concurrent writes
	for i := 0; i < 5; i++ {
		go func(id int) {
			orgInfo := OrgInfo{
				ID:   fmt.Sprintf("org_%d", id),
				Name: fmt.Sprintf("Org %d", id),
			}
			_ = cfg.AddOrganization(orgInfo)
			done <- true
		}(i)
	}

	// Concurrent reads
	for i := 0; i < 5; i++ {
		go func() {
			_ = cfg.GetCurrentOrg()
			_ = cfg.ListOrganizations()
			done <- true
		}()
	}

	// Wait for all goroutines
	for i := 0; i < 10; i++ {
		<-done
	}

	// Verify data integrity
	orgs := cfg.ListOrganizations()
	if len(orgs) < 5 {
		t.Errorf("Expected at least 5 organizations after concurrent adds, got %d", len(orgs))
	}
}
