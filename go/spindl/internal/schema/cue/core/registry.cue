// Registry and source handling
package core

import "strings"

// Registry configuration for fetching components
#Registry: {
	url!: string
	username?: string
	password?: string
}

// Source reference with optional registry
#SourceRef: {
	source!: string
	registry?: #Registry
}

// Normalize various source formats to canonical form
#NormalizeSource: {
	input: string
	
	output: string
	// Container images (keep as-is)
	if strings.HasPrefix(input, "ghcr.io/") || 
	   strings.HasPrefix(input, "docker.io/") ||
	   strings.HasPrefix(input, "registry.") {
		output: input
	}
	// Local files (ensure .wasm extension)
	if !strings.HasPrefix(input, "ghcr.io/") && 
	   !strings.HasPrefix(input, "docker.io/") &&
	   !strings.HasPrefix(input, "registry.") {
		if strings.HasSuffix(input, ".wasm") {
			output: input
		}
		if !strings.HasSuffix(input, ".wasm") {
			// For local files without extension, assume they need .wasm
			if strings.Contains(input, "./") || strings.HasPrefix(input, "/") {
				output: input + ".wasm"
			}
			// For bare names, treat as registry references
			if !strings.Contains(input, "./") && !strings.HasPrefix(input, "/") {
				output: "ghcr.io/fastertools/" + input + ":latest"
			}
		}
	}
}