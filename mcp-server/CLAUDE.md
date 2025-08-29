# FTL MCP Server Architecture

## Overview

The MCP (Model Context Protocol) server is **embedded within the main FTL binary**, not a standalone server. It provides programmatic access to FTL functionality for AI tools and the web console.

## Critical Architecture Understanding

### Single Binary Architecture
- **ONE BINARY**: Everything runs as part of the same `./bin/ftl` executable
- **Self-Reference**: Console calls itself via `os.Args[0]` with MCP subcommand
- **Tool Chain**: MCP tools call the same binary for filesystem operations

### Call Flow
```
Console (ftl dev console) 
    ↓ creates MCP client
    ↓ calls os.Args[0] ["dev", "mcp", "--ftl-path", "/path/to/self"]
    ↓
MCP Server (same binary, MCP mode)
    ↓ handles MCP tool calls
    ↓ calls configured FTL binary (usually self)  
    ↓
FTL Operations (same binary, CLI mode)
    ↓ performs actual filesystem changes
```

## MCP Tool Configuration

### Universal ToolConfig (`internal/types/tool_config.go`)
```go
type ToolConfig struct {
    FTLPath string `json:"ftl_path,omitempty"` // Path to FTL binary to use
}
```

### Command Line Integration
- **Console to MCP**: `--ftl-path /path/to/current/binary`
- **Test Mode**: `FTL_TEST_MODE=true` enables additional testing tools
- **Binary Path**: Configurable to support different binaries (production vs test builds)

## Tool Organization

### Core Tools (`tools/`)
- **lifecycle/**: `up`, `stop`, `status` - Process lifecycle management
- **operations/**: `build`, `components`, `logs` - Build and deployment operations  
- **process/**: Cross-platform process management utilities
- **testing/**: Test-specific tools (enabled with `--test-mode`)

### Registry System (`tools/registry.go`)
```go
// Default registry (uses "ftl" command)
registry := tools.NewRegistry(server)

// Configured registry (uses specific binary path)  
registry := tools.NewRegistryWithConfig(server, &types.ToolConfig{
    FTLPath: "/path/to/specific/ftl/binary",
})
```

## Integration Points

### Console Integration
The console (`internal/cli/dev_console.go`) integrates by:
1. Getting current executable path: `os.Executable()`
2. Creating MCP client with self-reference and binary path
3. Passing both test mode and binary configuration

### CLI Integration  
The CLI MCP command (`internal/cli/dev_mcp.go`) provides MCP server functionality embedded in the main binary.

## Tool Configuration Patterns

### Input Types (Planned)
```go
type FTLUpInput struct {
    types.ToolConfig                    // Universal config with FTLPath
    UpOptions       *cli.UpOptions     `json:"up_options,omitempty"`   
    ProjectPath     string             `json:"project_path"`
}
```

### Handler Pattern
```go
func (h *Handler) Handle(params map[string]interface{}) (map[string]interface{}, error) {
    // Parse input with existing CLI options
    var input FTLUpInput
    
    // Use configured FTL binary path
    ftlPath := input.GetFTLPath() // Defaults to "ftl" if not specified
    
    // Execute FTL command with proper binary
    cmd := exec.Command(ftlPath, args...)
    return executeAndFormat(cmd)
}
```

## Testing Architecture

### Test Mode Tools (`tools/testing/`)
- **Enabled**: Only when `FTL_TEST_MODE=true` 
- **Purpose**: Replace current TestHelpers functionality
- **Integration**: Test tools use configured binary path for test consistency

### Binary Path Management
- **Production**: Console passes its own path to ensure consistency
- **Testing**: Test mode can specify `./bin/ftl` to test specific builds
- **Environment Agnostic**: Tools don't know production vs test, just use provided path

## Key Design Principles

1. **Single Source of Truth**: One binary provides all functionality
2. **Path Configuration**: Binary path is configurable, not hardcoded
3. **Environment Agnostic**: Tools work the same in production/test
4. **Self-Contained**: No external dependencies or separate server processes
5. **Consistent Interface**: Same CLI options used everywhere (console, direct CLI, MCP tools)

## File Structure

```
mcp-server/
├── internal/
│   ├── types/          # Universal configuration types
│   ├── ftl/           # FTL command execution
│   ├── files/         # File management utilities  
│   ├── process/       # Process management
│   └── port/          # Port management
└── tools/
    ├── lifecycle/     # Process lifecycle tools
    ├── operations/    # Build/deploy operations
    ├── process/       # Process utilities
    ├── testing/       # Test-specific tools
    └── registry.go    # Tool registration and configuration
```

This architecture ensures that the console, direct CLI usage, and MCP tool operations all use the same underlying FTL functionality while maintaining configurability for different deployment scenarios.