package oci

// WASM OCI media types as defined by the CNCF TAG Runtime WASM OCI Artifact specification
// https://tag-runtime.cncf.io/wgs/wasm/deliverables/wasm-oci-artifact/
const (
	// WASMLayerMediaType is the media type for WASM layers
	WASMLayerMediaType = "application/wasm"

	// WASMConfigMediaType is the media type for WASM config blobs
	WASMConfigMediaType = "application/vnd.wasm.config.v0+json"

	// WASMManifestMediaType is the media type for WASM manifests
	WASMManifestMediaType = "application/vnd.oci.image.manifest.v1+json"

	// WASMArchitecture is the architecture field value for WASM artifacts
	WASMArchitecture = "wasm"

	// WASMOS is the OS field value for WASM components (wasip2)
	WASMOS = "wasip2"
)
