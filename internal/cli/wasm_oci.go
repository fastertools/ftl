package cli

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"

	v1 "github.com/google/go-containerregistry/pkg/v1"
	"github.com/google/go-containerregistry/pkg/v1/types"
)

// WASMConfig represents the config for a WASM OCI artifact
// This matches the structure expected by oci-wasm and Spin
type WASMConfig struct {
	Created      string   `json:"created"`
	Architecture string   `json:"architecture"`
	OS           string   `json:"os"`
	LayerDigests []string `json:"layerDigests"` // Critical field for Spin compatibility - must be camelCase!
	RootFS       struct {
		Type    string   `json:"type"`
		DiffIDs []string `json:"diff_ids"`
	} `json:"rootfs"`
	Config struct{} `json:"config"`
}

// wasmOCIImage implements v1.Image with proper WASM OCI config including layerDigests
type wasmOCIImage struct {
	wasmLayer v1.Layer
	config    []byte
	hashStr   string
}


// Layers returns the layers of the image
func (w *wasmOCIImage) Layers() ([]v1.Layer, error) {
	return []v1.Layer{w.wasmLayer}, nil
}

// MediaType returns the media type of the image
func (w *wasmOCIImage) MediaType() (types.MediaType, error) {
	return types.OCIManifestSchema1, nil
}

// Size returns the size of the image
func (w *wasmOCIImage) Size() (int64, error) {
	size, err := w.wasmLayer.Size()
	if err != nil {
		return 0, err
	}
	return size + int64(len(w.config)), nil
}

// ConfigName returns the config descriptor
func (w *wasmOCIImage) ConfigName() (v1.Hash, error) {
	h := sha256.Sum256(w.config)
	return v1.Hash{
		Algorithm: "sha256",
		Hex:       hex.EncodeToString(h[:]),
	}, nil
}

// ConfigFile returns the config file
func (w *wasmOCIImage) ConfigFile() (*v1.ConfigFile, error) {
	// We can't return the custom fields here, but we override RawConfigFile
	// to provide the actual config with layerDigests
	return &v1.ConfigFile{
		Architecture: "wasm",
		OS:           "wasip2",
		Config:       v1.Config{},
		RootFS: v1.RootFS{
			Type:    "layers",
			DiffIDs: []v1.Hash{{Algorithm: "sha256", Hex: w.hashStr}},
		},
	}, nil
}

// RawConfigFile returns the raw config file with layerDigests
func (w *wasmOCIImage) RawConfigFile() ([]byte, error) {
	Debug("Returning raw config with layerDigests: %s", string(w.config))
	return w.config, nil
}

// Digest returns the digest of the image
func (w *wasmOCIImage) Digest() (v1.Hash, error) {
	// Calculate manifest digest
	manifest, err := w.Manifest()
	if err != nil {
		return v1.Hash{}, err
	}
	
	manifestJSON, err := json.Marshal(manifest)
	if err != nil {
		return v1.Hash{}, err
	}
	
	h := sha256.Sum256(manifestJSON)
	return v1.Hash{
		Algorithm: "sha256",
		Hex:       hex.EncodeToString(h[:]),
	}, nil
}

// Manifest returns the manifest of the image
func (w *wasmOCIImage) Manifest() (*v1.Manifest, error) {
	layers := []v1.Descriptor{}
	
	// Add WASM layer descriptor
	layerDigest, err := w.wasmLayer.Digest()
	if err != nil {
		return nil, err
	}
	
	layerSize, err := w.wasmLayer.Size()
	if err != nil {
		return nil, err
	}
	
	layers = append(layers, v1.Descriptor{
		MediaType: "application/wasm",
		Size:      layerSize,
		Digest:    layerDigest,
	})
	
	// Create config descriptor
	configHash, err := w.ConfigName()
	if err != nil {
		return nil, err
	}
	
	return &v1.Manifest{
		SchemaVersion: 2,
		MediaType:     types.OCIManifestSchema1,
		Config: v1.Descriptor{
			MediaType: "application/vnd.wasm.config.v0+json",
			Size:      int64(len(w.config)),
			Digest:    configHash,
		},
		Layers: layers,
	}, nil
}

// RawManifest returns the raw manifest
func (w *wasmOCIImage) RawManifest() ([]byte, error) {
	manifest, err := w.Manifest()
	if err != nil {
		return nil, err
	}
	manifestJSON, err := json.Marshal(manifest)
	if err != nil {
		return nil, err
	}
	Debug("WASM Manifest JSON: %s", string(manifestJSON))
	return manifestJSON, nil
}

// LayerByDigest returns a layer by digest
func (w *wasmOCIImage) LayerByDigest(h v1.Hash) (v1.Layer, error) {
	layerDigest, err := w.wasmLayer.Digest()
	if err != nil {
		return nil, err
	}
	
	if layerDigest.String() == h.String() {
		return w.wasmLayer, nil
	}
	
	return nil, fmt.Errorf("layer not found: %s", h)
}

// LayerByDiffID returns a layer by diff ID
func (w *wasmOCIImage) LayerByDiffID(h v1.Hash) (v1.Layer, error) {
	diffID, err := w.wasmLayer.DiffID()
	if err != nil {
		return nil, err
	}
	
	if diffID.String() == h.String() {
		return w.wasmLayer, nil
	}
	
	return nil, fmt.Errorf("layer not found: %s", h)
}