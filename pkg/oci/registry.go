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
	"sync"
	"time"

	"github.com/google/go-containerregistry/pkg/authn"
	"github.com/google/go-containerregistry/pkg/name"
	v1 "github.com/google/go-containerregistry/pkg/v1"
	"github.com/google/go-containerregistry/pkg/v1/remote"
	"github.com/google/go-containerregistry/pkg/v1/static"
)

// WASMPuller handles pulling WASM components from OCI registries
type WASMPuller struct {
	cacheDir string
	mu       sync.Mutex
}

// NewWASMPuller creates a new WASM component puller
func NewWASMPuller() *WASMPuller {
	home := os.Getenv("HOME")
	cacheDir := filepath.Join(home, ".cache", "ftl", "wasm")
	if err := os.MkdirAll(cacheDir, 0750); err != nil {
		// Use temp dir as fallback if cache dir can't be created
		cacheDir = filepath.Join(os.TempDir(), "ftl-wasm-cache")
		_ = os.MkdirAll(cacheDir, 0750) // Best effort
	}

	return &WASMPuller{
		cacheDir: cacheDir,
	}
}

// NewWASMPullerWithCache creates a new WASM component puller with a custom cache directory
func NewWASMPullerWithCache(cacheDir string) *WASMPuller {
	return &WASMPuller{
		cacheDir: cacheDir,
	}
}

// Pull downloads a WASM component from a registry
// Parameters are now explicit instead of using a types package
func (p *WASMPuller) Pull(ctx context.Context, registry, packageName, version string) (string, error) {
	// Construct the OCI reference
	ref := fmt.Sprintf("%s/%s:%s", registry, packageName, version)

	// Parse the reference
	tag, err := name.ParseReference(ref)
	if err != nil {
		return "", fmt.Errorf("invalid reference %s: %w", ref, err)
	}

	// Pull the image
	img, err := remote.Image(tag, remote.WithAuthFromKeychain(authn.DefaultKeychain))
	if err != nil {
		return "", fmt.Errorf("failed to pull %s: %w", ref, err)
	}

	// Get the manifest to find the WASM layer
	manifest, err := img.Manifest()
	if err != nil {
		return "", fmt.Errorf("failed to get manifest: %w", err)
	}

	// Find the WASM layer (usually the first/only layer)
	if len(manifest.Layers) == 0 {
		return "", fmt.Errorf("no layers found in image")
	}

	// Get the first layer
	layers, err := img.Layers()
	if err != nil {
		return "", fmt.Errorf("failed to get layers: %w", err)
	}

	if len(layers) == 0 {
		return "", fmt.Errorf("no layers available")
	}

	layer := layers[0]

	// Get layer content
	reader, err := layer.Uncompressed()
	if err != nil {
		return "", fmt.Errorf("failed to get layer content: %w", err)
	}
	defer func() { _ = reader.Close() }()

	// Calculate hash for cache filename
	hash, err := layer.Digest()
	if err != nil {
		return "", fmt.Errorf("failed to get layer digest: %w", err)
	}

	// Create cache file path - hash.Hex is safe (it's a computed hash)
	cachePath := filepath.Clean(filepath.Join(p.cacheDir, hash.Hex+".wasm"))

	// Check if already cached
	if _, err := os.Stat(cachePath); err == nil {
		return cachePath, nil
	}

	// Write to cache
	p.mu.Lock()
	defer p.mu.Unlock()

	// Create temp file - hash.Hex is safe (it's a computed hash)
	tmpFile := filepath.Clean(cachePath + ".tmp")
	file, err := os.Create(tmpFile)
	if err != nil {
		return "", fmt.Errorf("failed to create cache file: %w", err)
	}

	_, err = io.Copy(file, reader)
	_ = file.Close()
	if err != nil {
		_ = os.Remove(tmpFile)
		return "", fmt.Errorf("failed to write WASM content: %w", err)
	}

	// Atomic rename
	if err := os.Rename(tmpFile, cachePath); err != nil {
		_ = os.Remove(tmpFile)
		return "", fmt.Errorf("failed to finalize cache file: %w", err)
	}

	return cachePath, nil
}

// WASMPusher handles pushing WASM components to OCI registries
type WASMPusher struct {
	auth *ECRAuth
}

// NewWASMPusher creates a new WASM component pusher
func NewWASMPusher(auth *ECRAuth) *WASMPusher {
	return &WASMPusher{auth: auth}
}

// Push uploads a WASM component to a registry as an OCI artifact
// Following the CNCF TAG Runtime WASM OCI Artifact specification
func (p *WASMPusher) Push(ctx context.Context, wasmPath, packageName, version string) error {
	// Clean the WASM file path
	wasmPath = filepath.Clean(wasmPath)

	// Read the WASM file
	wasmContent, err := os.ReadFile(wasmPath)
	if err != nil {
		return fmt.Errorf("failed to read WASM file: %w", err)
	}

	// Create and push the WASM OCI image
	img, err := p.createWASMImage(wasmContent, version)
	if err != nil {
		return fmt.Errorf("failed to create WASM image: %w", err)
	}

	// Construct the registry reference
	ref := fmt.Sprintf("%s/%s:%s", p.auth.Registry, packageName, version)

	// Parse the reference
	tag, err := name.ParseReference(ref)
	if err != nil {
		return fmt.Errorf("invalid reference %s: %w", ref, err)
	}

	// Create authenticator
	authConfig := authn.AuthConfig{
		Username: p.auth.Username,
		Password: p.auth.Password,
	}
	authenticator := authn.FromConfig(authConfig)

	// Push the image
	if err := remote.Write(tag, img, remote.WithAuth(authenticator)); err != nil {
		return fmt.Errorf("failed to push to registry: %w", err)
	}

	return nil
}

// createWASMImage creates a WASM OCI image from content
func (p *WASMPusher) createWASMImage(wasmContent []byte, version string) (v1.Image, error) {
	// Calculate SHA256 for the WASM content
	wasmHash := sha256.Sum256(wasmContent)
	wasmHashStr := hex.EncodeToString(wasmHash[:])

	// Create WASM layer with proper media type
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	// Create the config JSON with layerDigests field (critical for Spin)
	configData := WASMConfig{
		Created:      time.Now().UTC().Format(time.RFC3339),
		Architecture: WASMArchitecture,
		OS:           WASMOS,
		LayerDigests: []string{fmt.Sprintf("sha256:%s", wasmHashStr)},
	}
	configData.RootFS.Type = "layers"
	configData.RootFS.DiffIDs = []string{fmt.Sprintf("sha256:%s", wasmHashStr)}

	configJSON, err := json.Marshal(configData)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal config: %w", err)
	}

	// Create annotations for the manifest
	annotations := map[string]string{
		"org.opencontainers.image.version": version,
		"org.opencontainers.image.created": time.Now().UTC().Format(time.RFC3339),
	}

	// Create a custom WASM OCI image
	return &wasmOCIImage{
		wasmLayer:   wasmLayer,
		config:      configJSON,
		hashStr:     wasmHashStr,
		annotations: annotations,
	}, nil
}
