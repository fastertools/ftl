package schema

import _ "embed"

// EmbeddedSchemas contains all CUE schema definitions
var EmbeddedSchemas = map[string]string{
	"core/manifest.cue": CoreManifestSchema,
	"core/registry.cue": CoreRegistrySchema,
	"solutions/mcp.cue": MCPSolutionSchema,
}

// CoreManifestSchema defines the Spin manifest structure
//go:embed cue/core/manifest.cue
var CoreManifestSchema string

// CoreRegistrySchema defines source normalization and registry handling
//go:embed cue/core/registry.cue
var CoreRegistrySchema string

// MCPSolutionSchema defines the MCP application construct
//go:embed cue/solutions/mcp.cue
var MCPSolutionSchema string