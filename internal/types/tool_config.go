package types

// ToolConfig provides universal configuration for MCP tools that need to call FTL binary
type ToolConfig struct {
	FTLPath string `json:"ftl_path,omitempty"` // Optional custom FTL binary path, defaults to "ftl"
}

// GetFTLPath returns the FTL binary path, defaulting to "ftl" if not specified
func (tc *ToolConfig) GetFTLPath() string {
	if tc.FTLPath == "" {
		return "ftl"
	}
	return tc.FTLPath
}