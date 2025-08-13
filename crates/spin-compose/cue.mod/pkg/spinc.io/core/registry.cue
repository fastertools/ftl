// Registry reference parsing and handling
package core

import "strings"

// Parse a Docker/OCI registry reference into Spin registry format
#ParseRegistryRef: {
    input: string
    
    // Split by colon to separate version
    let parts = strings.Split(input, ":")
    let pathParts = strings.Split(parts[0], "/")
    
    // Determine registry, package, and version
    output: #RegistrySource & {
        if len(pathParts) >= 3 {
            // Format: registry.com/namespace/package:version
            registry: pathParts[0]
            package: strings.Join(pathParts[1:], "/")
            version: parts[1] | *"latest"
        }
        if len(pathParts) == 2 {
            // Format: namespace/package:version (assume Docker Hub)
            registry: "docker.io"
            package: parts[0]
            version: parts[1] | *"latest"
        }
        if len(pathParts) == 1 {
            // Format: package:version (assume Docker Hub library)
            registry: "docker.io"
            package: "library/" + pathParts[0]
            version: parts[1] | *"latest"
        }
    }
}

// Convert various source formats to Spin source
#NormalizeSource: {
    input: string | #RegistrySource
    
    output: string | #RegistrySource
    
    if (input & string) != _|_ {
        // Check if it's a file path or registry reference
        if strings.HasSuffix(input, ".wasm") || strings.HasPrefix(input, "./") || strings.HasPrefix(input, "/") {
            // Local file path
            output: input
        }
        if !strings.HasSuffix(input, ".wasm") && strings.Contains(input, ":") && strings.Contains(input, "/") {
            // Registry reference string - parse it
            let parsed = (#ParseRegistryRef & { "input": input }).output
            output: parsed
        }
    }
    
    if (input & #RegistrySource) != _|_ {
        // Already in registry format
        output: input
    }
}