// Package oci provides functionality for working with OCI (Open Container Initiative)
// registries and WASM (WebAssembly) artifacts according to the CNCF TAG Runtime
// WASM OCI Artifact specification.
//
// This package implements:
//   - WASM OCI image creation with proper layerDigests field for Spin compatibility
//   - Registry push/pull operations for WASM components
//   - ECR (Elastic Container Registry) authentication support
//   - Caching for pulled WASM artifacts
//
// The implementation follows the WASM OCI artifact specification used by tools like
// wkg (WebAssembly Package Manager) and Spin Framework, ensuring compatibility with
// the broader WASM ecosystem.
//
// Example usage:
//
//	// Push a WASM component to a registry
//	auth := &oci.ECRAuth{
//	    Registry: "123456.dkr.ecr.us-east-1.amazonaws.com",
//	    Username: "AWS",
//	    Password: "token",
//	}
//	pusher := oci.NewWASMPusher(auth)
//	err := pusher.Push(ctx, "component.wasm", "namespace/component", "1.0.0")
//
//	// Pull a WASM component from a registry
//	puller := oci.NewWASMPuller()
//	source := &types.RegistrySource{
//	    Registry: "ghcr.io",
//	    Package:  "org/component",
//	    Version:  "1.0.0",
//	}
//	wasmPath, err := puller.Pull(ctx, source)
package oci
