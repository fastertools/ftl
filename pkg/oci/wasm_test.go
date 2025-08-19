package oci

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"testing"
	"time"

	v1 "github.com/google/go-containerregistry/pkg/v1"
	"github.com/google/go-containerregistry/pkg/v1/static"
	"github.com/google/go-containerregistry/pkg/v1/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestWASMConfig(t *testing.T) {
	tests := []struct {
		name     string
		config   WASMConfig
		wantJSON string
	}{
		{
			name: "valid config with layerDigests",
			config: WASMConfig{
				Created:      "2024-01-01T00:00:00Z",
				Architecture: WASMArchitecture,
				OS:           WASMOS,
				LayerDigests: []string{"sha256:abcd1234"},
			},
			wantJSON: `{"created":"2024-01-01T00:00:00Z","architecture":"wasm","os":"wasip2","layerDigests":["sha256:abcd1234"],"rootfs":{"type":"","diff_ids":null},"config":{}}`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jsonData, err := json.Marshal(tt.config)
			require.NoError(t, err)
			assert.JSONEq(t, tt.wantJSON, string(jsonData))

			// Verify layerDigests field is in camelCase
			assert.Contains(t, string(jsonData), `"layerDigests"`)
			assert.NotContains(t, string(jsonData), `"layer_digests"`)
		})
	}
}

func TestWASMOCIImage_Layers(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	img := &wasmOCIImage{
		wasmLayer:   wasmLayer,
		config:      []byte("{}"),
		hashStr:     "test",
		annotations: nil,
	}

	layers, err := img.Layers()
	require.NoError(t, err)
	assert.Len(t, layers, 1)
	assert.Equal(t, wasmLayer, layers[0])
}

func TestWASMOCIImage_MediaType(t *testing.T) {
	img := &wasmOCIImage{}

	mediaType, err := img.MediaType()
	require.NoError(t, err)
	assert.Equal(t, types.OCIManifestSchema1, mediaType)
}

func TestWASMOCIImage_Size(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)
	config := []byte(`{"test":"config"}`)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
		config:    config,
		hashStr:   "test",
	}

	size, err := img.Size()
	require.NoError(t, err)

	layerSize, _ := wasmLayer.Size()
	expectedSize := layerSize + int64(len(config))
	assert.Equal(t, expectedSize, size)
}

func TestWASMOCIImage_ConfigName(t *testing.T) {
	config := []byte(`{"test":"config"}`)
	expectedHash := sha256.Sum256(config)
	expectedHex := hex.EncodeToString(expectedHash[:])

	img := &wasmOCIImage{
		config: config,
	}

	hash, err := img.ConfigName()
	require.NoError(t, err)
	assert.Equal(t, "sha256", hash.Algorithm)
	assert.Equal(t, expectedHex, hash.Hex)
}

func TestWASMOCIImage_ConfigFile(t *testing.T) {
	hashStr := "abcd1234"
	img := &wasmOCIImage{
		hashStr: hashStr,
	}

	configFile, err := img.ConfigFile()
	require.NoError(t, err)

	assert.Equal(t, WASMArchitecture, configFile.Architecture)
	assert.Equal(t, WASMOS, configFile.OS)
	assert.Equal(t, "layers", configFile.RootFS.Type)
	assert.Len(t, configFile.RootFS.DiffIDs, 1)
	assert.Equal(t, "sha256", configFile.RootFS.DiffIDs[0].Algorithm)
	assert.Equal(t, hashStr, configFile.RootFS.DiffIDs[0].Hex)
}

func TestWASMOCIImage_RawConfigFile(t *testing.T) {
	config := []byte(`{"layerDigests":["sha256:test"]}`)
	img := &wasmOCIImage{
		config: config,
	}

	rawConfig, err := img.RawConfigFile()
	require.NoError(t, err)
	assert.Equal(t, config, rawConfig)
}

func TestWASMOCIImage_Manifest(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)
	config := []byte(`{"test":"config"}`)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
		config:    config,
		hashStr:   "test",
	}

	manifest, err := img.Manifest()
	require.NoError(t, err)

	assert.Equal(t, int64(2), manifest.SchemaVersion)
	assert.Equal(t, types.OCIManifestSchema1, manifest.MediaType)
	assert.Equal(t, types.MediaType(WASMConfigMediaType), manifest.Config.MediaType)
	assert.Equal(t, int64(len(config)), manifest.Config.Size)

	assert.Len(t, manifest.Layers, 1)
	assert.Equal(t, types.MediaType(WASMLayerMediaType), manifest.Layers[0].MediaType)
}

func TestWASMOCIImage_RawManifest(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)
	config := []byte(`{"test":"config"}`)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
		config:    config,
		hashStr:   "test",
	}

	rawManifest, err := img.RawManifest()
	require.NoError(t, err)

	// Parse the raw manifest to verify it's valid JSON
	var manifest v1.Manifest
	err = json.Unmarshal(rawManifest, &manifest)
	require.NoError(t, err)
	assert.Equal(t, int64(2), manifest.SchemaVersion)
}

func TestWASMOCIImage_Digest(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)
	config := []byte(`{"test":"config"}`)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
		config:    config,
		hashStr:   "test",
	}

	digest, err := img.Digest()
	require.NoError(t, err)
	assert.Equal(t, "sha256", digest.Algorithm)
	assert.NotEmpty(t, digest.Hex)
}

func TestWASMOCIImage_LayerByDigest(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
	}

	layerDigest, err := wasmLayer.Digest()
	require.NoError(t, err)

	// Test finding existing layer
	foundLayer, err := img.LayerByDigest(layerDigest)
	require.NoError(t, err)
	assert.Equal(t, wasmLayer, foundLayer)

	// Test with non-existent digest
	nonExistentDigest := v1.Hash{
		Algorithm: "sha256",
		Hex:       "nonexistent",
	}
	_, err = img.LayerByDigest(nonExistentDigest)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "layer not found")
}

func TestWASMOCIImage_LayerByDiffID(t *testing.T) {
	wasmContent := []byte("test wasm content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
	}

	diffID, err := wasmLayer.DiffID()
	require.NoError(t, err)

	// Test finding existing layer
	foundLayer, err := img.LayerByDiffID(diffID)
	require.NoError(t, err)
	assert.Equal(t, wasmLayer, foundLayer)

	// Test with non-existent diff ID
	nonExistentDiffID := v1.Hash{
		Algorithm: "sha256",
		Hex:       "nonexistent",
	}
	_, err = img.LayerByDiffID(nonExistentDiffID)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "layer not found")
}

func TestWASMOCIImage_IntegrationWithRealConfig(t *testing.T) {
	// Create a realistic WASM config
	wasmContent := []byte("actual wasm binary content here")
	wasmHash := sha256.Sum256(wasmContent)
	wasmHashStr := hex.EncodeToString(wasmHash[:])

	configData := WASMConfig{
		Created:      time.Now().UTC().Format(time.RFC3339),
		Architecture: WASMArchitecture,
		OS:           WASMOS,
		LayerDigests: []string{fmt.Sprintf("sha256:%s", wasmHashStr)},
	}
	configData.RootFS.Type = "layers"
	configData.RootFS.DiffIDs = []string{fmt.Sprintf("sha256:%s", wasmHashStr)}

	configJSON, err := json.Marshal(configData)
	require.NoError(t, err)

	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	img := &wasmOCIImage{
		wasmLayer: wasmLayer,
		config:    configJSON,
		hashStr:   wasmHashStr,
	}

	// Verify the config contains layerDigests in camelCase
	rawConfig, err := img.RawConfigFile()
	require.NoError(t, err)
	assert.Contains(t, string(rawConfig), `"layerDigests"`)
	assert.Contains(t, string(rawConfig), wasmHashStr)

	// Verify manifest is properly formed
	manifest, err := img.Manifest()
	require.NoError(t, err)
	assert.Equal(t, types.MediaType(WASMConfigMediaType), manifest.Config.MediaType)
	assert.Len(t, manifest.Layers, 1)
	assert.Equal(t, types.MediaType(WASMLayerMediaType), manifest.Layers[0].MediaType)

	// Verify we can retrieve the layer
	layers, err := img.Layers()
	require.NoError(t, err)
	assert.Len(t, layers, 1)

	reader, err := layers[0].Uncompressed()
	require.NoError(t, err)
	defer reader.Close()

	retrievedContent, err := io.ReadAll(reader)
	require.NoError(t, err)
	assert.Equal(t, wasmContent, retrievedContent)
}

func TestWASMOCIImage_ImplementsV1Image(t *testing.T) {
	// This test ensures our wasmOCIImage properly implements v1.Image interface
	var _ v1.Image = &wasmOCIImage{}
}

func TestWASMConstants(t *testing.T) {
	// Verify constants match expected values from WASM OCI spec
	assert.Equal(t, "application/wasm", WASMLayerMediaType)
	assert.Equal(t, "application/vnd.wasm.config.v0+json", WASMConfigMediaType)
	assert.Equal(t, "application/vnd.oci.image.manifest.v1+json", WASMManifestMediaType)
	assert.Equal(t, "wasm", WASMArchitecture)
	assert.Equal(t, "wasip2", WASMOS)
}

func TestWASMOCIImage_ManifestWithAnnotations(t *testing.T) {
	// Test that annotations are properly included in the manifest
	wasmContent := []byte("test content")
	wasmLayer := static.NewLayer(wasmContent, WASMLayerMediaType)

	annotations := map[string]string{
		"org.opencontainers.image.version": "1.2.3",
		"org.opencontainers.image.created": "2024-01-01T00:00:00Z",
		"custom.annotation":                "value",
	}

	img := &wasmOCIImage{
		wasmLayer:   wasmLayer,
		config:      []byte("{}"),
		hashStr:     "test",
		annotations: annotations,
	}

	manifest, err := img.Manifest()
	require.NoError(t, err)
	assert.Equal(t, annotations, manifest.Annotations)
}
