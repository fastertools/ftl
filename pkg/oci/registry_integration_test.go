package oci

import (
	"context"
	"fmt"
	"io"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/google/go-containerregistry/pkg/authn"
	"github.com/google/go-containerregistry/pkg/name"
	"github.com/google/go-containerregistry/pkg/registry"
	"github.com/google/go-containerregistry/pkg/v1/remote"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	ftltypes "github.com/fastertools/ftl-cli/pkg/types"
)

func TestWASMPuller_Pull_Integration(t *testing.T) {
	// Create a test registry
	s := httptest.NewServer(registry.New())
	defer s.Close()

	regURL := strings.TrimPrefix(s.URL, "http://")

	// Create test WASM content
	wasmContent := []byte{0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00} // Valid WASM header

	// Push a test WASM artifact using our pusher
	pusher := NewWASMPusher(&ECRAuth{
		Registry: regURL,
		Username: "test",
		Password: "test",
	})

	ctx := context.Background()

	// Create a temp file for pushing
	tmpFile, err := os.CreateTemp("", "test*.wasm")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	err = os.WriteFile(tmpFile.Name(), wasmContent, 0644)
	require.NoError(t, err)

	// Push the artifact
	err = pusher.Push(ctx, tmpFile.Name(), "test/component", "1.0.0")
	require.NoError(t, err)

	// Now test pulling it back
	tempCacheDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempCacheDir)

	source := &ftltypes.RegistrySource{
		Registry: regURL,
		Package:  "test/component",
		Version:  "1.0.0",
	}

	// First pull - should download
	wasmPath, err := puller.Pull(ctx, source)
	require.NoError(t, err)
	assert.FileExists(t, wasmPath)
	assert.Contains(t, wasmPath, ".wasm")

	// Verify content
	pulledContent, err := os.ReadFile(wasmPath)
	require.NoError(t, err)
	assert.Equal(t, wasmContent, pulledContent)

	// Second pull - should use cache
	wasmPath2, err := puller.Pull(ctx, source)
	require.NoError(t, err)
	assert.Equal(t, wasmPath, wasmPath2)

	// Verify the cache was actually used by checking file times
	stat1, err := os.Stat(wasmPath)
	require.NoError(t, err)
	stat2, err := os.Stat(wasmPath2)
	require.NoError(t, err)
	assert.Equal(t, stat1.ModTime(), stat2.ModTime())
}

func TestWASMPuller_Pull_ManifestError(t *testing.T) {
	// Test error handling when manifest retrieval fails
	tempCacheDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempCacheDir)

	ctx := context.Background()

	// Use an invalid registry that will fail
	source := &ftltypes.RegistrySource{
		Registry: "invalid.registry.test",
		Package:  "test/component",
		Version:  "1.0.0",
	}

	_, err := puller.Pull(ctx, source)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to pull")
}

func TestWASMPuller_Pull_LayerReadError(t *testing.T) {
	tempCacheDir := t.TempDir()
	puller := NewWASMPullerWithCache(tempCacheDir)

	ctx := context.Background()

	// Test with malformed registry URL that will fail during pull
	source := &ftltypes.RegistrySource{
		Registry: "notaregistry.invalid",
		Package:  "test/component",
		Version:  "1.0.0",
	}

	_, err := puller.Pull(ctx, source)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to pull")
}

func TestWASMPuller_CacheDirectoryCreation(t *testing.T) {
	// Test with HOME env var set
	t.Run("with HOME", func(t *testing.T) {
		tempHome := t.TempDir()
		oldHome := os.Getenv("HOME")
		os.Setenv("HOME", tempHome)
		defer os.Setenv("HOME", oldHome)

		puller := NewWASMPuller()
		assert.NotNil(t, puller)
		expectedCacheDir := filepath.Join(tempHome, ".cache", "ftl", "wasm")
		assert.Equal(t, expectedCacheDir, puller.cacheDir)
		assert.DirExists(t, expectedCacheDir)
	})

	// Test with HOME unset (will use .cache/ftl/wasm relative to current dir)
	t.Run("without HOME", func(t *testing.T) {
		oldHome := os.Getenv("HOME")
		os.Unsetenv("HOME")
		defer os.Setenv("HOME", oldHome)

		puller := NewWASMPuller()
		assert.NotNil(t, puller)
		// When HOME is not set, it creates .cache/ftl/wasm relative to current directory
		assert.Contains(t, puller.cacheDir, ".cache/ftl/wasm")
	})
}

func TestWASMPusher_Push_Integration(t *testing.T) {
	// Create a test registry
	s := httptest.NewServer(registry.New())
	defer s.Close()

	regURL := strings.TrimPrefix(s.URL, "http://")

	pusher := NewWASMPusher(&ECRAuth{
		Registry: regURL,
		Username: "test",
		Password: "test",
	})

	ctx := context.Background()

	// Create a test WASM file with specific content
	tmpFile, err := os.CreateTemp("", "test*.wasm")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	wasmContent := []byte("test wasm content for push")
	err = os.WriteFile(tmpFile.Name(), wasmContent, 0644)
	require.NoError(t, err)

	// Push the artifact
	err = pusher.Push(ctx, tmpFile.Name(), "namespace/component", "2.0.0")
	require.NoError(t, err)

	// Verify we can pull it back
	ref, err := name.ParseReference(fmt.Sprintf("%s/namespace/component:2.0.0", regURL))
	require.NoError(t, err)

	img, err := remote.Image(ref, remote.WithAuthFromKeychain(authn.DefaultKeychain))
	require.NoError(t, err)

	// Verify the image has the correct structure
	layers, err := img.Layers()
	require.NoError(t, err)
	assert.Len(t, layers, 1)

	// Verify layer content
	reader, err := layers[0].Uncompressed()
	require.NoError(t, err)
	defer reader.Close()

	pulledContent, err := io.ReadAll(reader)
	require.NoError(t, err)
	assert.Equal(t, wasmContent, pulledContent)

	// Verify manifest has annotations
	manifest, err := img.Manifest()
	require.NoError(t, err)
	assert.NotNil(t, manifest.Annotations)
	assert.Equal(t, "2.0.0", manifest.Annotations["org.opencontainers.image.version"])
	assert.NotEmpty(t, manifest.Annotations["org.opencontainers.image.created"])
}

func TestWASMPuller_Pull_CacheCorruption(t *testing.T) {
	tempCacheDir := t.TempDir()

	// Create a test registry
	s := httptest.NewServer(registry.New())
	defer s.Close()

	regURL := strings.TrimPrefix(s.URL, "http://")

	// Push a test artifact
	wasmContent := []byte("valid wasm content")
	pusher := NewWASMPusher(&ECRAuth{
		Registry: regURL,
		Username: "test",
		Password: "test",
	})

	tmpFile, err := os.CreateTemp("", "test*.wasm")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	err = os.WriteFile(tmpFile.Name(), wasmContent, 0644)
	require.NoError(t, err)

	ctx := context.Background()
	err = pusher.Push(ctx, tmpFile.Name(), "test/cache", "1.0.0")
	require.NoError(t, err)

	// Create puller and pull once to populate cache
	puller := NewWASMPullerWithCache(tempCacheDir)
	source := &ftltypes.RegistrySource{
		Registry: regURL,
		Package:  "test/cache",
		Version:  "1.0.0",
	}

	wasmPath1, err := puller.Pull(ctx, source)
	require.NoError(t, err)

	// Corrupt the cache file
	err = os.WriteFile(wasmPath1, []byte("corrupted"), 0644)
	require.NoError(t, err)

	// Pull again - should still use corrupted cache since we check existence only
	wasmPath2, err := puller.Pull(ctx, source)
	require.NoError(t, err)
	assert.Equal(t, wasmPath1, wasmPath2)

	// Verify it returns the corrupted file
	content, err := os.ReadFile(wasmPath2)
	require.NoError(t, err)
	assert.Equal(t, []byte("corrupted"), content)
}

func TestWASMPuller_Pull_CacheWriteError(t *testing.T) {
	// Create a cache dir that we'll make read-only
	tempCacheDir := t.TempDir()

	// Create a test registry and push content
	s := httptest.NewServer(registry.New())
	defer s.Close()

	regURL := strings.TrimPrefix(s.URL, "http://")

	wasmContent := []byte("test content")
	pusher := NewWASMPusher(&ECRAuth{
		Registry: regURL,
		Username: "test",
		Password: "test",
	})

	tmpFile, err := os.CreateTemp("", "test*.wasm")
	require.NoError(t, err)
	defer os.Remove(tmpFile.Name())

	err = os.WriteFile(tmpFile.Name(), wasmContent, 0644)
	require.NoError(t, err)

	ctx := context.Background()
	err = pusher.Push(ctx, tmpFile.Name(), "test/readonly", "1.0.0")
	require.NoError(t, err)

	// Make cache dir read-only
	err = os.Chmod(tempCacheDir, 0555)
	require.NoError(t, err)
	defer os.Chmod(tempCacheDir, 0755) // Restore permissions for cleanup

	// Try to pull - should fail when trying to write to cache
	puller := NewWASMPullerWithCache(tempCacheDir)
	source := &ftltypes.RegistrySource{
		Registry: regURL,
		Package:  "test/readonly",
		Version:  "1.0.0",
	}

	_, err = puller.Pull(ctx, source)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "failed to create cache file")
}
