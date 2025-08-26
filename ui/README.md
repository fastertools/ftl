# FTL UI Module

This directory contains the FTL web UI implementation using CUE modules for configuration and code generation.

## Structure

```
ui/
├── cue.mod/          # CUE module definition
│   └── module.cue    # Module: github.com/fastertools/ftl/ui
├── schema/           # CUE schema definitions
│   ├── colors.cue    # Color and style constants
│   ├── strings.cue   # UI string constants
│   └── utils.cue     # Utility schemas
├── gen/              # Generated Go code (do not edit)
│   ├── strings_gen.go
│   └── styles_gen.go
├── templates/        # Templ templates
└── generate_v2.go    # CUE → Go generator
```

## Architecture

The UI follows a three-layer architecture:

1. **CUE Layer** (`schema/`): Defines contracts and constants
   - Type-safe configuration
   - Single source of truth for UI strings and styles
   - Validation and constraints

2. **Go Layer** (`gen/`): Generated runtime code
   - Strongly-typed constants
   - Used by handlers and templates
   - Auto-generated from CUE schemas

3. **Templ Layer** (`templates/`): HTML rendering
   - Component-based templates
   - Uses generated constants
   - Compiled to Go code

## Development

### Regenerating UI Constants

After modifying CUE files in `schema/`:

```bash
make generate-ui
```

Or directly:

```bash
cd ui && go run generate_v2.go
```

### Adding New Constants

1. Edit the appropriate CUE file in `schema/`
2. Run generation: `make generate-ui`
3. Use the generated constants in Go code:

```go
import ui "github.com/fastertools/ftl/ui/gen"

// Use generated constants
status := ui.StatusRunning
color := ui.Colors.Primary.BG
```

## CUE Module

This UI package is a proper CUE module (`github.com/fastertools/ftl/ui`), enabling:

- Versioned schema management
- Import/export of schemas
- Dependency management
- Type-safe configuration

## Generated Code

The `gen/` directory contains auto-generated Go code. **Do not edit these files manually** - they will be overwritten on the next generation.

Generated from:
- `colors.cue` → Style constants and CSS classes
- `strings.cue` → UI text, messages, and labels

## Safety Features

The generator includes:
- File locking to prevent race conditions
- Atomic writes for safe generation
- Validation of CUE schemas before generation
- Go format validation of generated code