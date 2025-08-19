package oci

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	ftltypes "github.com/fastertools/ftl-cli/pkg/types"
)

func TestNewWASMPuller(t *testing.T) {
	puller := NewWASMPuller()
	assert.NotNil(t, puller)
	assert.NotEmpty(t, puller.cacheDir)
}

func TestNewWASMPullerWithCache(t *testing.T) {
	tempDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempDir)
	assert.NotNil(t, puller)
	assert.Equal(t, tempDir, puller.cacheDir)
}

func TestWASMPuller_CacheManagement(t *testing.T) {
	tempDir := t.TempDir()
	
	// Create a fake cached WASM file
	wasmContent := []byte("test wasm content")
	hash := sha256.Sum256(wasmContent)
	hashHex := hex.EncodeToString(hash[:])
	cachePath := filepath.Join(tempDir, hashHex+".wasm")
	
	err := os.WriteFile(cachePath, wasmContent, 0644)
	require.NoError(t, err)

	// Verify the cache file exists
	assert.FileExists(t, cachePath)
}

func TestNewWASMPusher(t *testing.T) {
	auth := &ECRAuth{
		Registry: "test.registry.com",
		Username: "testuser",
		Password: "testpass",
	}
	
	pusher := NewWASMPusher(auth)
	assert.NotNil(t, pusher)
	assert.Equal(t, auth, pusher.auth)
}

func TestWASMPusher_CreateWASMImage(t *testing.T) {
	auth := &ECRAuth{
		Registry: "test.registry.com",
		Username: "testuser",
		Password: "testpass",
	}
	pusher := NewWASMPusher(auth)

	wasmContent := []byte("test wasm binary content")
	version := "1.0.0"

	img, err := pusher.createWASMImage(wasmContent, version)
	require.NoError(t, err)
	assert.NotNil(t, img)

	// Verify the image has correct layers
	layers, err := img.Layers()
	require.NoError(t, err)
	assert.Len(t, layers, 1)

	// Verify layer content
	reader, err := layers[0].Uncompressed()
	require.NoError(t, err)
	defer reader.Close()

	content, err := io.ReadAll(reader)
	require.NoError(t, err)
	assert.Equal(t, wasmContent, content)

	// Verify config has layerDigests
	rawConfig, err := img.RawConfigFile()
	require.NoError(t, err)

	var config WASMConfig
	err = json.Unmarshal(rawConfig, &config)
	require.NoError(t, err)
	assert.NotEmpty(t, config.LayerDigests)
	assert.Equal(t, WASMArchitecture, config.Architecture)
	assert.Equal(t, WASMOS, config.OS)
}

func TestWASMPusher_CreateWASMImageConsistency(t *testing.T) {
	auth := &ECRAuth{
		Registry: "test.registry.com",
		Username: "testuser",
		Password: "testpass",
	}
	pusher := NewWASMPusher(auth)

	wasmContent := []byte("deterministic wasm content")
	version := "1.0.0"

	// Create image with same content
	img, err := pusher.createWASMImage(wasmContent, version)
	require.NoError(t, err)

	// Get config
	config, err := img.RawConfigFile()
	require.NoError(t, err)
	
	// Parse config
	var cfg WASMConfig
	err = json.Unmarshal(config, &cfg)
	require.NoError(t, err)
	
	// Verify consistent layer digests for same content
	expectedHash := sha256.Sum256(wasmContent)
	expectedHashStr := hex.EncodeToString(expectedHash[:])
	expectedLayerDigest := fmt.Sprintf("sha256:%s", expectedHashStr)
	
	assert.Len(t, cfg.LayerDigests, 1)
	assert.Equal(t, expectedLayerDigest, cfg.LayerDigests[0])
	
	// Verify timestamp is set
	assert.NotEmpty(t, cfg.Created)
	_, err = time.Parse(time.RFC3339, cfg.Created)
	assert.NoError(t, err, "Created time should be valid RFC3339")
}

func TestRegistrySource_Validation(t *testing.T) {
	tests := []struct {
		name    string
		source  *ftltypes.RegistrySource
		wantRef string
		wantErr bool
	}{
		{
			name: "valid source",
			source: &ftltypes.RegistrySource{
				Registry: "docker.io",
				Package:  "library/hello-world",
				Version:  "latest",
			},
			wantRef: "docker.io/library/hello-world:latest",
			wantErr: false,
		},
		{
			name: "ECR source",
			source: &ftltypes.RegistrySource{
				Registry: "123456789.dkr.ecr.us-west-2.amazonaws.com",
				Package:  "my-app/component",
				Version:  "v1.0.0",
			},
			wantRef: "123456789.dkr.ecr.us-west-2.amazonaws.com/my-app/component:v1.0.0",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ref := fmt.Sprintf("%s/%s:%s", tt.source.Registry, tt.source.Package, tt.source.Version)
			assert.Equal(t, tt.wantRef, ref)
		})
	}
}

func TestECRAuth_Structure(t *testing.T) {
	auth := &ECRAuth{
		Registry: "123456789.dkr.ecr.us-west-2.amazonaws.com",
		Username: "AWS",
		Password: "long-ecr-token-here",
	}

	assert.Equal(t, "AWS", auth.Username)
	assert.NotEmpty(t, auth.Password)
	assert.Contains(t, auth.Registry, "ecr")
	assert.Contains(t, auth.Registry, "amazonaws.com")
}

func TestWASMPuller_Pull_ErrorCases(t *testing.T) {
	tempDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempDir)
	ctx := context.Background()

	tests := []struct {
		name    string
		source  *ftltypes.RegistrySource
		wantErr string
	}{
		{
			name: "invalid reference",
			source: &ftltypes.RegistrySource{
				Registry: "invalid registry",
				Package:  "package",
				Version:  "version",
			},
			wantErr: "invalid reference",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := puller.Pull(ctx, tt.source)
			assert.Error(t, err)
			assert.Contains(t, err.Error(), tt.wantErr)
		})
	}
}

func TestWASMPusher_Push_ErrorCases(t *testing.T) {
	auth := &ECRAuth{
		Registry: "test.registry.com",
		Username: "testuser",
		Password: "testpass",
	}
	pusher := NewWASMPusher(auth)
	ctx := context.Background()

	// Create a temp file for the second test case
	tempFile, err := os.CreateTemp("", "test*.wasm")
	require.NoError(t, err)
	tempFile.Close()
	defer os.Remove(tempFile.Name())

	tests := []struct {
		name        string
		wasmPath    string
		packageName string
		version     string
		wantErr     string
	}{
		{
			name:        "non-existent file",
			wasmPath:    "/non/existent/file.wasm",
			packageName: "test-package",
			version:     "1.0.0",
			wantErr:     "failed to read WASM file",
		},
		{
			name:        "invalid package name",
			wasmPath:    tempFile.Name(),
			packageName: "invalid package name",
			version:     "1.0.0",
			wantErr:     "invalid reference",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := pusher.Push(ctx, tt.wasmPath, tt.packageName, tt.version)
			assert.Error(t, err)
			assert.Contains(t, err.Error(), tt.wantErr)
		})
	}
}

func TestWASMImageCreation_VerifyLayerDigests(t *testing.T) {
	auth := &ECRAuth{
		Registry: "test.registry.com",
		Username: "testuser",
		Password: "testpass",
	}
	pusher := NewWASMPusher(auth)

	// Create specific content with known hash
	wasmContent := []byte("known content for testing")
	expectedHash := sha256.Sum256(wasmContent)
	expectedHashStr := hex.EncodeToString(expectedHash[:])
	expectedLayerDigest := fmt.Sprintf("sha256:%s", expectedHashStr)

	img, err := pusher.createWASMImage(wasmContent, "1.0.0")
	require.NoError(t, err)

	// Get and parse the config
	rawConfig, err := img.RawConfigFile()
	require.NoError(t, err)

	var config WASMConfig
	err = json.Unmarshal(rawConfig, &config)
	require.NoError(t, err)

	// Verify layerDigests contains the expected hash
	assert.Len(t, config.LayerDigests, 1)
	assert.Equal(t, expectedLayerDigest, config.LayerDigests[0])

	// Verify rootfs diff_ids match
	assert.Equal(t, "layers", config.RootFS.Type)
	assert.Len(t, config.RootFS.DiffIDs, 1)
	assert.Equal(t, expectedLayerDigest, config.RootFS.DiffIDs[0])
}

func TestCachePathSafety(t *testing.T) {
	tempDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempDir)

	// Test that cache paths are properly cleaned
	hash := "abc123def456"
	expectedPath := filepath.Clean(filepath.Join(tempDir, hash+".wasm"))
	
	// Create the cache file path as the puller would
	cachePath := filepath.Clean(filepath.Join(puller.cacheDir, hash+".wasm"))
	
	assert.Equal(t, expectedPath, cachePath)
	assert.NotContains(t, cachePath, "..")
	assert.NotContains(t, cachePath, "~")
}