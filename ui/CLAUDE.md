# FTL UI Architecture

## Overview

The FTL UI is the web-based development console that provides real-time monitoring of build processes, logs, component status, and project management through an HTMX-powered interface.

## Architecture Summary

This is a **self-contained console** that runs as part of the main FTL binary:

1. **Console Launch**: `ftl dev console` starts the web server
2. **MCP Integration**: Console creates MCP client that calls back to itself via internal stdio calls OR HTMX endpoints 
3. **Tool Operations**: HTMX frontend calls MCP tools for build/deploy/status operations
4. **File System Changes**: MCP tools call the same FTL binary for actual filesystem operations

## Key Components

### Web Server (`internal/cli/dev_console.go`)
- Serves HTMX-powered interface
- Creates MCP client pointing to self: `mcpClient := mcpclient.NewClientWithArgs(os.Args[0], []string{"dev", "mcp"})`
- Handles HTMX endpoints for UI interactions

### HTMX Handlers (`internal/handlers/`)
- Translate UI actions to MCP tool calls
- Return HTML fragments for dynamic updates
- Simple approach: UI passes minimal params, MCP tools handle complexity

### Templates (`ui/templates/`)
- Templ-based type-safe HTML generation
- JavaScript interactions using templ script templates
- **CRITICAL**: Use `event.target` instead of `this` in script functions

## UI Standards

### Templ JavaScript Patterns
```go
// ✅ CORRECT - Use event.target for DOM access
script handleHover(event templ.JSExpression, bgColor, borderColor string) {
    event.target.style.backgroundColor = bgColor;
    event.target.style.borderColor = borderColor;
}

// ❌ WRONG - this is undefined in templ scripts  
script badHover(color string) {
    this.style.backgroundColor = color; // ERROR
}
```

### HTMX Integration
- Keep handlers simple: "ftl up" in project location
- MCP tools accept comprehensive options but UI uses defaults
- Let MCP tools handle complexity, UI stays minimal

## Directory Structure

```
ui/
├── templates/          # Templ template files
├── gen/               # Generated constants from CUE
├── schema/            # CUE modules for UI constants  
├── e2e/               # Browser tests
└── static/            # CSS and assets
```

## Build Process

The UI is built as part of the main `make build` process:
1. `generate-ui` - Generates constants from CUE modules
2. `generate-templ` - Compiles templ templates to Go
3. `generate-css` - Builds Tailwind CSS
4. Main Go build includes everything

## Testing

- **Browser Tests**: Playwright-based in `ui/e2e/`
- **Integration Tests**: Test HTMX endpoints through MCP tools
- **Component Tests**: Individual templ template testing

## Console Architecture Flow

```
Browser --> HTMX --> Console Handlers --> MCP Client --> MCP Tools --> FTL Binary --> Filesystem
   ↑                                           ↓
   └──── HTML Fragments ←────────── MCP Response ←─────────┘
```

This self-referential architecture allows the console to provide a web interface while using the same binary for all operations, ensuring consistency between console actions and direct CLI usage.